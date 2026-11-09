# Coverage measurement. The counting itself is done as each file is read and
# as each line runs, and the bookkeeping here decides what a run reports.
module Coverage
  MODES = [:lines, :branches, :methods, :eval, :oneshot_lines]

  # The modes a run reports under their own names, in the order Ruby lists
  # them.
  REPORTED_MODES = [:lines, :branches, :methods]

  module_function

  def supported?(mode)
    unless mode.is_a?(Symbol)
      raise TypeError, "wrong argument type #{mode.class} (expected Symbol)"
    end
    MODES.include?(mode)
  end

  def running?
    @running ? true : false
  end

  def start(options = nil)
    raise RuntimeError, "coverage measurement is already setup" if running?
    unless options.nil? || options == :all || options.is_a?(Hash)
      raise TypeError, "no implicit conversion of #{options.class} into Hash"
    end
    if options.is_a?(Hash) && options[:lines] && options[:oneshot_lines]
      raise RuntimeError, "cannot enable lines and oneshot_lines simultaneously"
    end

    if options == :all
      modes = REPORTED_MODES
      by_mode = true
      eval_too = true
    elsif options.is_a?(Hash)
      modes = REPORTED_MODES.select { |mode| options[mode] }
      by_mode = !modes.empty? || options[:oneshot_lines] ? true : false
      modes = [:lines] if modes.empty?
      eval_too = options[:eval] ? true : false
    else
      modes = [:lines]
      by_mode = false
      eval_too = false
    end

    @running = true
    __coverage_start__ by_mode, modes, eval_too
    nil
  end

  def setup(options = nil)
    start options
  end

  def result(**options)
    raise RuntimeError, "coverage measurement is not enabled" unless running?
    stop_given = options.key? :stop
    clear_given = options.key? :clear
    stop = stop_given ? options[:stop] : !clear_given
    clear = clear_given ? options[:clear] : stop
    if stop && stop_given && !(clear_given && clear)
      warn "warning: stop implies clear"
      clear = true
    end
    counted = __coverage_branches__
    answer = __with_method_ends__(__with_branches__(__coverage_result__(clear), counted))
    if stop
      @running = false
      __coverage_stop__
    end
    answer
  end

  def peek_result
    raise RuntimeError, "coverage measurement is not enabled" unless running?
    __with_method_ends__(__with_branches__(__coverage_result__(false), __coverage_branches__))
  end

  # The report with each method `define_method` made from a block keyed to
  # where that block closes, which is read from the file.
  def __with_method_ends__(answer)
    answer.each do |path, report|
      next unless report.is_a?(Hash) && report[:methods].is_a?(Hash)
      next unless report[:methods].each_key.any? { |key| key[4].nil? }

      closings = __block_closings__(path)
      report[:methods] = report[:methods].to_h do |key, count|
        found = key[4].nil? && closings[[key[2], key[3]]]
        found ? [[*key[0, 4], *found], count] : [key, count]
      end
    end
    answer
  end

  # Where each block in a file closes, by the line and column it opens at.
  def __block_closings__(path)
    require "prism"
    parsed = Prism.parse_file(path)
    closings = {}
    pending = [parsed.value]
    until pending.empty?
      node = pending.pop
      if node.is_a?(Prism::BlockNode) || node.is_a?(Prism::LambdaNode)
        place = node.location
        closings[[place.start_line, place.start_column]] = [place.end_line, place.end_column]
      end
      pending.concat(node.compact_child_nodes)
    end
    closings
  rescue SystemCallError
    {}
  end

  # The report with each file's branches laid out as Ruby lays them out, from
  # the branch sites the run counted.
  def __with_branches__(answer, counted)
    answer.each do |path, report|
      next unless report.is_a?(Hash) && report.key?(:branches)

      require "coverage/branch_layout"
      held = counted.find { |file, _| file == path }
      report[:branches] = BranchLayout.call(path, held ? held.last : [])
    end
    answer
  end

  # The lines a file would report before any of it runs: 0 where a line
  # carries something to run and nil elsewhere.
  def line_stub(path)
    __coverage_line_stub__ File.read(path)
  end
end
