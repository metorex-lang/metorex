# RubyVM::InstructionSequence for a program metorex reads. Metorex runs the
# syntax tree rather than instructions, so an instruction sequence holds the
# source it was compiled from and the node that source parses to. Its
# listing names the nodes MRI's compiler would have read, not YARV
# instructions.
class RubyVM
  class InstructionSequence
    # The compile options MRI answers, kept as given.
    COMPILE_OPTIONS = {
      inline_const_cache: true, peephole_optimization: true, tailcall_optimization: false,
      specialized_instruction: true, operands_unification: true, instructions_unification: false,
      debug_frozen_string_literal: false, coverage_enabled: true, debug_level: 0, frozen_string_literal: nil
    }.freeze

    # What `to_binary` writes ahead of the dumped sequence.
    BINARY_HEADER = "METOREX-ISEQ\0"

    class << self
      def __compile__(source, file, path, line)
        source = source.read if source.respond_to?(:read)
        source = source.to_str
        node = begin
          RubyVM::AbstractSyntaxTree.parse(source)
        rescue SyntaxError => error
          raise SyntaxError, "#{file}:#{error_line(error, line)}: #{error.message}"
        end
        __build__(source: source, path: file, absolute_path: path, label: "<compiled>", base_label: "<compiled>",
                  first_lineno: line, type: :top, node: node)
      end

      # The sequences MRI's heap holds for the Ruby files loaded so far: the
      # top level of each and every method, class and block written in it.
      def __loaded__
        files = [$PROGRAM_NAME, *$LOADED_FEATURES].select { |file| file.end_with?(".rb") && File.file?(file) }
        found = []
        files.uniq.each do |file|
          outline = file_outline(file)
          outlined(file, File.expand_path(file), outline, found) if outline
        end
        found
      end

      # The sequence a `script_compiled` event reports for `source`, compiled
      # as an eval at `path` from `line`, which names no file on disk.
      def __script__(source, path, line)
        outline = __sequence_outline__(source)
        return __compile__(source, path, nil, line) if outline.nil?

        sequence = outlined(path, nil, outline, [])
        sequence.instance_variable_set(:@first_lineno, line)
        sequence.instance_variable_set(:@type, :eval)
        sequence.instance_variable_set(:@label, "<compiled>")
        sequence.instance_variable_set(:@base_label, "<compiled>")
        sequence
      end

      # The sequence an outline of `file` describes, after adding it and
      # those of the scopes written in it to `found`.
      def outlined(file, absolute_path, outline, found)
        type, label, first_lineno = outline
        # The top level of a file starts on line 0, and only the program's
        # own is `<main>`.
        if type == :top
          first_lineno = 0
          label = "<top (required)>" unless file == $PROGRAM_NAME
        end
        base = label.sub(/\Ablock (\(\d+ levels\) )?in /, "")
        sequence = __build__(source: nil, path: file, absolute_path: absolute_path, label: label,
                             base_label: base, first_lineno: first_lineno, type: type, node: nil, outline: outline)
        found << sequence
        sequence.instance_variable_set(:@children, outline[6].map { |child| outlined(file, absolute_path, child, found) })
        sequence
      end

      # The line a parse error names, counted from the line the source
      # starts on.
      def error_line(error, line)
        (error.instance_variable_get(:@line) || 1) + line - 1
      end

      def __compile_file__(file)
        source = File.read(file)
        node = begin
          RubyVM::AbstractSyntaxTree.parse(source)
        rescue SyntaxError => error
          raise SyntaxError, "#{file}:#{error_line(error, 1)}: #{error.message}"
        end
        __build__(source: source, path: file, absolute_path: File.expand_path(file), label: "<main>",
                  base_label: "<main>", first_lineno: 1, type: :top, node: node)
      end

      # The sequence a Proc or a Method was compiled into, or nil for one
      # with no Ruby source.
      def __of__(body)
        unless body.is_a?(Proc) || body.is_a?(Method) || body.is_a?(UnboundMethod)
          raise TypeError, "wrong argument type #{body.class} (expected Proc or Method)"
        end
        path, = body.source_location
        return nil if path.nil? || !File.file?(path)

        node = RubyVM::AbstractSyntaxTree.of(body)
        return nil if node.nil?

        source = File.read(path)
        if body.is_a?(Proc)
          enclosing = enclosing_label(RubyVM::AbstractSyntaxTree.parse(source), node)
          block_sequence(source, path, File.expand_path(path), node, enclosing)
        else
          method_sequence(source, path, File.expand_path(path), node)
        end
      end

      # The sequence a debugger frame runs. A method's is named by the method
      # alone and starts at its `def`, a block's at the block, and the
      # program's at line 0.
      def __frame_sequence__(location)
        path = location.path
        label = location.label.to_s
        block = label.start_with?("block ")
        prefix, owner = block ? label.split(" in ", 2) : [nil, label]
        base = owner.start_with?("<") ? owner : owner.split(/[#.]/).last
        label = block ? "#{prefix} in #{base}" : base
        type = block ? :block : (base == "<main>" ? :top : :method)
        holder = base == "<main>" && !block ? nil : frame_holder(path, location.lineno, block ? nil : base)
        if holder
          sequence = outlined(path, location.absolute_path, holder, [])
          sequence.instance_variable_set(:@label, label)
          sequence.instance_variable_set(:@base_label, base)
          return sequence
        end
        line = base == "<main>" && !block ? 0 : location.lineno
        __build__(source: nil, path: path, absolute_path: location.absolute_path, label: label, base_label: base,
                  first_lineno: line, type: type, node: nil)
      end

      # The outline of the method named `name`, or of the innermost block,
      # holding line `line` of the file at `path`, or nil where the file
      # holds none.
      def frame_holder(path, line, name)
        outline = file_outline(path)
        return nil if outline.nil?

        holders = []
        collect = lambda do |scope|
          type, label, first, last, = scope
          holds = first <= line && line <= last
          holders << scope if holds && (name ? type == :method && label == name : type == :block)
          scope[6].each { |child| collect.call(child) }
        end
        collect.call(outline)
        holders.last
      end

      # The outline of the file at `path`, read again only once the file
      # changes.
      def file_outline(path)
        return nil unless path && File.file?(path)

        @file_outlines ||= {}
        stamp = File.mtime(path)
        held = @file_outlines[path]
        return held[1] if held && held[0] == stamp

        outline = __sequence_outline__(File.read(path))
        @file_outlines[path] = [stamp, outline]
        outline
      end

      def __build__(**fields)
        sequence = __allocate_instance__(self)
        fields.each { |name, value| sequence.instance_variable_set(:"@#{name}", value) }
        sequence
      end

      def method_sequence(source, path, absolute_path, node)
        name = node.type == :DEFN ? node.children[0] : node.children[1]
        __build__(source: source, path: path, absolute_path: absolute_path, label: name.to_s, base_label: name.to_s,
                  first_lineno: node.first_lineno, type: :method, node: node)
      end

      def block_sequence(source, path, absolute_path, node, enclosing)
        levels, base = enclosing
        label = levels > 1 ? "block (#{levels} levels) in #{base}" : "block in #{base}"
        __build__(source: source, path: path, absolute_path: absolute_path, label: label, base_label: base,
                  first_lineno: node.first_lineno, type: :block, node: node)
      end

      # How many blocks deep `target` sits, and the label of the method,
      # class or program that holds them.
      def enclosing_label(root, target)
        path = []
        found = search(root, target, path)
        return [1, "<main>"] unless found

        # The block itself is the first level.
        levels = 1
        path.reverse_each do |ancestor|
          case ancestor.type
          when :ITER, :LAMBDA then levels += 1
          when :DEFN then return [levels, ancestor.children[0].to_s]
          when :DEFS then return [levels, ancestor.children[1].to_s]
          when :CLASS then return [levels, "<class:#{constant_name(ancestor.children[0])}>"]
          when :MODULE then return [levels, "<module:#{constant_name(ancestor.children[0])}>"]
          when :SCLASS then return [levels, "singleton class"]
          end
        end
        [levels, "<main>"]
      end

      def search(node, target, path)
        return false unless node.is_a?(RubyVM::AbstractSyntaxTree::Node)
        return true if same_place?(node, target)

        path.push(node)
        node.children.each do |child|
          items = child.is_a?(Array) ? child : [child]
          return true if items.any? { |item| search(item, target, path) }
        end
        path.pop
        false
      end

      def constant_name(path) = path.children.last.to_s

      # Whether two nodes, from separate parses of one file, are the same.
      def same_place?(node, other)
        node.type == other.type && node.first_lineno == other.first_lineno && node.first_column == other.first_column &&
          node.last_lineno == other.last_lineno && node.last_column == other.last_column
      end

      private :__compile__, :__compile_file__, :__loaded__, :__script__, :outlined, :__of__, :__frame_sequence__, :__build__, :error_line, :frame_holder, :file_outline,
              :method_sequence, :block_sequence, :enclosing_label, :search, :constant_name, :same_place?
    end

    attr_reader :path, :absolute_path, :label, :base_label, :first_lineno

    def inspect = "<RubyVM::InstructionSequence:#{@label}@#{@path}:#{@first_lineno}>"

    # Runs a sequence compiled from a whole program, at the top level.
    def eval
      raise TypeError, "Not a toplevel InstructionSequence" unless @type == :top

      TOPLEVEL_BINDING.eval(@source, @path, @first_lineno)
    end

    # MRI keeps no script lines for a sequence compiled from a string.
    def script_lines = nil

    # The line each statement starts on, which MRI reports as a line event,
    # with the call and return of a method or block around them.
    def trace_points
      return outlined_trace_points if @outline
      return [] if @node.nil?

      points = statement_lines(body_node).map { |line| [line, :line] }
      case @type
      when :method then [[@node.first_lineno, :call], *points, [@node.last_lineno, :return]]
      when :block then [[@node.first_lineno, :b_call], *points, [@node.last_lineno, :b_return]]
      else points
      end
    end

    # The sequences of the methods, classes and blocks written directly in
    # this one.
    def each_child
      raise LocalJumpError, "no block given" unless block_given?

      (@children || children_of(body_node)).each { |child| yield child }
      self
    end

    # The sequence as MRI's SimpleDataFormat Array, with the instructions
    # standing for the nodes of the syntax tree.
    def to_a
      scope = scope_node
      locals = scope ? scope.children[0].compact : []
      [
        "YARVInstructionSequence/SimpleDataFormat", 4, 0, 1,
        { arg_size: argument_count, local_size: locals.size, stack_max: 1, node_id: @node&.node_id,
          code_location: code_location, node_ids: [], parser: :metorex },
        @label, @path, @absolute_path, @first_lineno, @type, locals, parameters, [], instructions
      ]
    end

    # A listing of the nodes the sequence holds, under MRI's heading.
    def disasm
      heading = "== disasm: #<ISeq:#{@label}@#{@path}:#{@first_lineno} (#{code_location[0]},#{code_location[1]})-(#{code_location[2]},#{code_location[3]})>\n"
      lines = instructions.select { |item| item.is_a?(Array) }.each_with_index.map do |(name, *operands), index|
        format("%04d %-38s\n", index, [name, *operands.map(&:inspect)].join(" ").rstrip)
      end
      heading + lines.join
    end
    alias disassemble disasm

    # The sequence written as bytes `load_from_binary` reads back.
    def to_binary(extra = nil)
      BINARY_HEADER + Marshal.dump([@source, @path, @absolute_path, @label, @base_label, @first_lineno, @type, extra])
    end

    private_class_method def self.__load_from_binary__(binary)
      raise ArgumentError, "unknown binary format" unless binary.start_with?(BINARY_HEADER)

      source, path, absolute_path, label, base_label, line, type, = Marshal.load(binary.byteslice(BINARY_HEADER.bytesize..))
      node = source && RubyVM::AbstractSyntaxTree.parse(source)
      __build__(source: source, path: path, absolute_path: absolute_path, label: label, base_label: base_label,
                first_lineno: line, type: type, node: node)
    end

    private_class_method def self.__load_from_binary_extra_data__(binary)
      raise ArgumentError, "unknown binary format" unless binary.start_with?(BINARY_HEADER)

      Marshal.load(binary.byteslice(BINARY_HEADER.bytesize..)).last
    end

    private

    # The SCOPE a method, block or program runs.
    def scope_node
      return nil if @node.nil?
      return @node if @node.type == :SCOPE
      return @node.children.last if %i[DEFN DEFS].include?(@node.type)
      return @node.children[1] if @node.type == :ITER

      @node.children[0] if @node.type == :LAMBDA
    end

    def body_node = scope_node&.children&.last

    def argument_count
      arguments = scope_node&.children&.at(1)
      return 0 unless arguments.is_a?(RubyVM::AbstractSyntaxTree::Node)

      arguments.children[0].to_i + arguments.children[4].to_i
    end

    # The parameters as MRI's Hash names them: the count of leading ones
    # and the rest, keyword and block parameters by name.
    def parameters
      arguments = scope_node&.children&.at(1)
      return {} unless arguments.is_a?(RubyVM::AbstractSyntaxTree::Node)

      lead, _, _, _, post, _, rest, keywords, _, block = arguments.children
      described = {}
      described[:lead_num] = lead if lead.positive?
      described[:post_num] = post if post.positive?
      described[:rest_start] = lead if rest.is_a?(Symbol) && rest != :NODE_SPECIAL_EXCESSIVE_COMMA
      described[:keyword] = keyword_names(keywords) if keywords
      described[:use_block] = true if block
      described
    end

    # The parameters as `Method#parameters` lists them, with a Proc's
    # required ones optional.
    def __parameter_list__(proc)
      return @outline[5].map { |kind, name| [proc && kind == :req ? :opt : kind, name] } if @outline

      scope = scope_node
      arguments = scope&.children&.at(1)
      return [] unless arguments.is_a?(RubyVM::AbstractSyntaxTree::Node)

      table = scope.children[0]
      lead, _, optional, first_post, post, _, rest, keywords, keyword_rest, block = arguments.children
      required = proc ? :opt : :req
      listed = table.first(lead).map { |name| [required, name] }
      while optional.is_a?(RubyVM::AbstractSyntaxTree::Node)
        listed << [:opt, optional.children[0].children[0]]
        optional = optional.children[1]
      end
      listed << [:rest, rest] if rest.is_a?(Symbol) && rest != :NODE_SPECIAL_EXCESSIVE_COMMA
      listed.concat(table[table.index(first_post), post].map { |name| [required, name] }) if post.positive?
      while keywords.is_a?(RubyVM::AbstractSyntaxTree::Node)
        name, value = keywords.children[0].children
        listed << [value == :NODE_SPECIAL_REQUIRED_KEYWORD ? :keyreq : :key, name]
        keywords = keywords.children[1]
      end
      listed << [:keyrest, keyword_rest.children[0]] if keyword_rest.is_a?(RubyVM::AbstractSyntaxTree::Node)
      listed << [:block, block] if block
      listed
    end

    # The events an outlined sequence reports: a line for each statement, with
    # the call and return of a method or block, or the class and end of a
    # class body, around them.
    def outlined_trace_points
      type, _, first, last, lines = @outline
      points = lines.map { |line| [line, :line] }
      # A method or block is entered on the line its first statement starts,
      # after that line's own event.
      opening = points.first(1)
      rest = points.drop(1)
      entered = lines.first || first
      case type
      when :method then [*opening, [entered, :call], *rest, [last, :return]]
      when :block then [*opening, [entered, :b_call], *rest, [last, :b_return]]
      when :class then [[first, :class], *points, [last, :end]]
      else points
      end
    end

    def keyword_names(chain)
      names = []
      while chain.is_a?(RubyVM::AbstractSyntaxTree::Node)
        names << chain.children[0].children[0]
        chain = chain.children[1]
      end
      names
    end

    def code_location
      return [@outline[2], 0, @outline[3], 0] if @outline
      return [@first_lineno, 0, @first_lineno, 0] if @node.nil?

      [@node.first_lineno, @node.first_column, @node.last_lineno, @node.last_column]
    end

    # The nodes in the order MRI's compiler reads them: the line of each
    # statement, then each node before the nodes it holds, as [type,
    # values...].
    def instructions
      if @outline
        return trace_points.flat_map { |line, event| [line, :"RUBY_EVENT_#{event.upcase}"] } << [:leave]
      end

      listed = []
      statements(body_node).each do |statement|
        listed << statement.first_lineno << :RUBY_EVENT_LINE
        walk(statement) { |node| listed << [node.type.downcase, *node.children.grep_v(RubyVM::AbstractSyntaxTree::Node).grep_v(Array)] }
      end
      listed << [:leave]
    end

    # Visits a node and those it holds. A method, class or block is a
    # sequence of its own, so only a block's call is visited.
    def walk(node, &visit)
      return unless node.is_a?(RubyVM::AbstractSyntaxTree::Node)

      visit.call(node)
      return walk(node.children[0], &visit) if node.type == :ITER
      return if %i[DEFN DEFS CLASS MODULE SCLASS LAMBDA].include?(node.type)

      node.children.each do |child|
        (child.is_a?(Array) ? child : [child]).each { |item| walk(item, &visit) }
      end
    end

    def statements(body)
      return [] unless body.is_a?(RubyVM::AbstractSyntaxTree::Node)

      body.type == :BLOCK ? body.children : [body]
    end

    # The lines the statements of a body start on.
    def statement_lines(body) = statements(body).map(&:first_lineno).uniq

    def children_of(node)
      found = []
      collect = lambda do |value|
        case value
        when RubyVM::AbstractSyntaxTree::Node
          case value.type
          when :DEFN, :DEFS
            found << self.class.send(:method_sequence, @source, @path, @absolute_path, value)
          when :CLASS, :MODULE
            name = "<#{value.type == :CLASS ? "class" : "module"}:#{self.class.send(:constant_name, value.children[0])}>"
            found << self.class.send(:__build__, source: @source, path: @path, absolute_path: @absolute_path, label: name,
                                          base_label: name, first_lineno: value.first_lineno, type: :class, node: value)
          when :ITER, :LAMBDA
            found << self.class.send(:block_sequence, @source, @path, @absolute_path, value, [1, @label])
          else
            value.children.each { |child| collect.call(child) }
          end
        when Array then value.each { |item| collect.call(item) }
        end
      end
      collect.call(node)
      found
    end
  end
end
