require "prism"

module Coverage
  # Where each branch of a file stands and how often it was taken, keyed the
  # way Ruby keys them: a condition as [kind, id, first line, first column,
  # last line, last column], with each of its arms keyed the same way. The
  # places come from Prism's reading of the file, and the counts from the
  # branch sites the run counted, found by kind and the line their keyword
  # stands on.
  class BranchLayout
    def self.call(path, sites)
      parsed = Prism.parse_file(path)
      return {} if parsed.failure?

      layout = new(sites)
      layout.walk(parsed.value)
      layout.branches
    rescue SystemCallError
      {}
    end

    attr_reader :branches

    def initialize(sites)
      @branches = {}
      @next_id = 0
      @sites = Hash.new { |held, key| held[key] = [] }
      sites.each { |kind, line, arms| @sites[[kind, line]] << arms.map { |arm| arm[2] } }
      @chains = {}.compare_by_identity
    end

    def walk(node)
      return if node.nil?

      case node
      when Prism::IfNode, Prism::UnlessNode
        # The condition is read first, so a site inside it is numbered
        # ahead of this one.
        return folded_arm(node) if folded(node.predicate)

        walk(node.predicate)
        node.is_a?(Prism::IfNode) ? if_node(node) : unless_node(node)
        node.compact_child_nodes.each { |child| walk(child) unless child.equal?(node.predicate) }
        return
      when Prism::CaseNode, Prism::CaseMatchNode then case_node(node)
      when Prism::WhileNode then loop_node(node, :while)
      when Prism::UntilNode then loop_node(node, :until)
      when Prism::CallNode
        # The call a safe navigation is made on is read first, so a site
        # inside it is numbered ahead of this one.
        if node.safe_navigation?
          walk(node.receiver)
          safe_navigation(node)
          node.compact_child_nodes.each { |child| walk(child) unless child.equal?(node.receiver) }
          return
        end
      end
      node.compact_child_nodes.each { |child| walk(child) }
    end

    private

    # A condition that is a constant keeps only the arm it takes, and names
    # no branch of its own.
    def folded_arm(node)
      truthy = folded(node.predicate) == :truthy
      if node.is_a?(Prism::IfNode)
        walk(truthy ? node.statements : node.subsequent)
      else
        walk(truthy ? node.else_clause : node.statements)
      end
    end

    def if_node(node)
      counts = if_counts(node)
      add(:if, node,
        [[:then, node.statements || end_of(node.predicate), counts[0]],
         [:else, if_else_span(node), counts[1]]])
    end

    def if_else_span(node)
      following = node.subsequent
      return node if following.nil?
      return following if following.is_a?(Prism::IfNode)

      following.statements || following
    end

    # The counts of an `if` and its `elsif`s come from one site that has an
    # arm for each, so the k-th condition takes the k-th arm and is passed
    # over as often as the arms after it ran.
    def if_counts(node)
      if node.if_keyword_loc&.slice == "elsif"
        arms, index = @chains.fetch(node, [nil, 0])
      else
        line = (node.if_keyword_loc || node.location).start_line
        arms = take(:if, line)
        index = 0
      end
      following = node.subsequent
      @chains[following] = [arms, index + 1] if following.is_a?(Prism::IfNode)
      return [0, 0] if arms.nil?

      [arms[index] || 0, arms.drop(index + 1).sum]
    end

    def unless_node(node)
      taken, passed = take(:unless, node.keyword_loc.start_line) || [0, 0]
      following = node.else_clause
      add(:unless, node,
        [[:else, following ? (following.statements || node) : node, passed || 0],
         [:then, node.statements || node, taken || 0]])
    end

    def case_node(node)
      kind = node.is_a?(Prism::CaseMatchNode) ? :in : :when
      counts = take(:case, node.case_keyword_loc.start_line) || []
      arms = node.conditions.each_with_index.map do |clause, index|
        [kind, clause.statements || clause, counts[index] || 0]
      end
      following = node.else_clause
      span = following.nil? ? node : (following.statements || following)
      arms << [:else, span, counts[node.conditions.size] || 0]
      add(:case, node, arms)
    end

    def loop_node(node, kind)
      counts = take(:while, node.keyword_loc.start_line) || []
      add(kind, node, [[:body, node.statements || node, counts[0] || 0]])
    end

    def safe_navigation(node)
      last = node.closing_loc || node.arguments || node.message_loc
      span = [node.location.start_line, node.location.start_column, last.end_line, last.end_column]
      counts = take(:"&.", node.call_operator_loc.start_line) || []
      add(:"&.", span, [[:then, span, counts[0] || 0], [:else, span, counts[1] || 0]])
    end

    def take(kind, line)
      @sites[[kind, line]].shift
    end

    def add(kind, span, arms)
      condition = key(kind, span)
      @branches[condition] = arms.to_h { |arm_kind, arm_span, count| [key(arm_kind, arm_span), count] }
    end

    def key(kind, span)
      id = @next_id
      @next_id += 1
      [kind, id, *place(span)]
    end

    def place(span)
      return span if span.is_a?(Array)

      found = span.respond_to?(:location) ? span.location : span
      [found.start_line, found.start_column, found.end_line, found.end_column]
    end

    def end_of(node)
      found = node.location
      [found.end_line, found.end_column, found.end_line, found.end_column]
    end

    # Literals Ruby reads a condition from without running anything.
    CONSTANT_CONDITIONS = [
      Prism::IntegerNode, Prism::FloatNode, Prism::RationalNode, Prism::ImaginaryNode,
      Prism::SymbolNode, Prism::StringNode, Prism::TrueNode, Prism::FalseNode, Prism::NilNode,
      Prism::SourceLineNode, Prism::SourceEncodingNode, Prism::LambdaNode
    ].freeze
    FALSY_CONDITIONS = [Prism::FalseNode, Prism::NilNode].freeze
    # Conditions that stop being read as constants once they are wrapped in
    # parentheses.
    OPAQUE_IN_PARENTHESES = [Prism::NilNode, Prism::StringNode, Prism::LambdaNode].freeze
    LITERAL_LEAVES = [
      Prism::IntegerNode, Prism::FloatNode, Prism::RationalNode, Prism::ImaginaryNode,
      Prism::StringNode, Prism::SymbolNode, Prism::TrueNode, Prism::FalseNode, Prism::NilNode,
      Prism::SourceLineNode, Prism::SourceFileNode, Prism::SourceEncodingNode,
      Prism::RegularExpressionNode
    ].freeze
    DISCARDED_READS = [
      Prism::SelfNode, Prism::LocalVariableReadNode, Prism::InstanceVariableReadNode,
      Prism::DefinedNode
    ].freeze

    # :truthy or :falsy for a condition Ruby folds away, nil otherwise.
    def folded(node)
      unwrapped = unwrap(node)
      return nil unless CONSTANT_CONDITIONS.any? { |type| unwrapped.instance_of?(type) }
      return nil unless unwrapped.equal?(node) || OPAQUE_IN_PARENTHESES.none? { |type| unwrapped.instance_of?(type) }

      FALSY_CONDITIONS.any? { |type| unwrapped.instance_of?(type) } ? :falsy : :truthy
    end

    # The last statement of parentheses whose earlier statements do nothing.
    def unwrap(node)
      current = node
      while current.instance_of?(Prism::ParenthesesNode)
        body = current.body
        break unless body.instance_of?(Prism::StatementsNode)

        statements = body.body
        break unless statements[0...-1].all? { |leading| discarded?(leading) }

        current = statements.last
      end
      current
    end

    def discarded?(node)
      return true if literal?(node)
      return true if DISCARDED_READS.any? { |type| node.instance_of?(type) }

      node.instance_of?(Prism::ParenthesesNode) &&
        node.body.instance_of?(Prism::StatementsNode) &&
        node.body.body.all? { |statement| discarded?(statement) }
    end

    def literal?(node)
      return true if LITERAL_LEAVES.any? { |type| node.instance_of?(type) }

      case node
      when Prism::ArrayNode then node.elements.all? { |element| literal?(element) }
      when Prism::HashNode
        node.elements.all? { |element| element.instance_of?(Prism::AssocNode) && literal?(element.key) && literal?(element.value) }
      when Prism::RangeNode then [node.left, node.right].all? { |bound| bound.nil? || literal?(bound) }
      else false
      end
    end
  end
end
