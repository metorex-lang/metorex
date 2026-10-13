require "erb"

module Rex
  module Core
    module Formatters
      # `--format html`: the run as one HTML page, in RSpec's layout.
      class HtmlFormatter < BaseFormatter
        Formatters.register self, :start, :example_group_started, :start_dump,
                            :example_started, :example_passed, :example_failed,
                            :example_pending, :dump_summary

        def initialize(output)
          super(output)
          @failed_examples = []
          @example_group_number = 0
          @example_number = 0
          @header_red = nil
          @printer = HtmlPrinter.new(output)
        end

        def start(notification)
          super
          @printer.print_html_start
          @printer.flush
        end

        def example_group_started(notification)
          super
          @example_group_red = false
          @example_group_number += 1
          @printer.print_example_group_end unless @example_group_number == 1
          @printer.print_example_group_start(@example_group_number, notification.group.description,
                                             notification.group.parent_groups.size)
          @printer.flush
        end

        def start_dump(_notification)
          @printer.print_example_group_end
          @printer.flush
        end

        def example_started(_notification)
          @example_number += 1
        end

        def example_passed(passed)
          @printer.move_progress(percent_done)
          @printer.print_example_passed(passed.example.description, passed.example.execution_result.run_time)
          @printer.flush
        end

        def example_failed(failure)
          @failed_examples << failure.example
          unless @header_red
            @header_red = true
            @printer.make_header_red
          end
          unless @example_group_red
            @example_group_red = true
            @printer.make_example_group_header_red(@example_group_number)
          end
          @printer.move_progress(percent_done)

          example = failure.example
          exception = failure.exception
          message_lines = failure.fully_formatted_lines(nil)
          # The first two lines are the description and a blank line.
          exception_details = if exception
                                {
                                  message: message_lines.drop(2).join("\n"),
                                  backtrace: failure.formatted_backtrace.join("\n")
                                }
                              end
          extra = extra_failure_content(failure)
          @printer.print_example_failed(example.execution_result.pending_fixed?, example.description,
                                        example.execution_result.run_time, @failed_examples.size,
                                        exception_details, extra == "" ? false : extra)
          @printer.flush
        end

        def example_pending(pending)
          example = pending.example
          @printer.make_header_yellow unless @header_red
          @printer.make_example_group_header_yellow(@example_group_number) unless @example_group_red
          @printer.move_progress(percent_done)
          @printer.print_example_pending(example.description, example.execution_result.pending_message)
          @printer.flush
        end

        def dump_summary(summary)
          @printer.print_summary(summary.duration, summary.example_count, summary.failure_count, summary.pending_count)
          @printer.flush
        end

        private

        def percent_done
          return 100.0 unless @example_count > 0

          ((@example_number.to_f / @example_count * 1000).to_i / 10.0).to_f
        end

        # The source around the failing line, for a subclass to replace with
        # other content.
        def extra_failure_content(failure)
          formatter = Rex.configuration.backtrace_formatter
          backtrace = (failure.exception.backtrace || []).reject { |line| formatter.exclude?(line) }
                                                         .map { |line| formatter.backtrace_line(line) }
          @snippet_extractor ||= HtmlSnippetExtractor.new
          "    <pre class=\"ruby\"><code>#{@snippet_extractor.snippet(backtrace)}</code></pre>"
        end
      end

      # The lines around a failure, numbered, with the failing one marked.
      class HtmlSnippetExtractor
        def snippet(backtrace)
          raw_code, line = snippet_for(backtrace[0])
          highlighted = %(#{raw_code}\n<span class="comment"># Install the coderay gem to get syntax highlighting</span>)
          post_process(highlighted, line)
        end

        def snippet_for(error_line)
          if error_line =~ /(.*):(\d+)/
            file = Regexp.last_match(1)
            line = Regexp.last_match(2).to_i
            [lines_around(file, line), line]
          else
            ["# Couldn't get snippet for #{error_line}", 1]
          end
        end

        # Two lines above the failing one and one below.
        def lines_around(file, line)
          return "# Couldn't get snippet for #{file}" unless File.file?(file)

          lines = File.read(file).split("\n")
          min = [0, line - 3].max
          max = [line + 1, lines.length - 1].min
          lines[min..max].join("\n")
        end

        def post_process(highlighted, offending_line)
          highlighted.split("\n").each_with_index.map do |line, index|
            new_line = "<span class=\"linenum\">#{offending_line + index - 2}</span>#{line}"
            index == 2 ? "<span class=\"offending\">#{new_line}</span>" : new_line
          end.join("\n")
        end
      end

      class HtmlPrinter
        include ERB::Util

        def initialize(output)
          @output = output
        end

        def print_html_start
          @output.puts HTML_HEADER
          @output.puts REPORT_HEADER
        end

        def print_example_group_end
          @output.puts "  </dl>"
          @output.puts "</div>"
        end

        def print_example_group_start(group_id, description, number_of_parents)
          @output.puts "<div id=\"div_group_#{group_id}\" class=\"example_group passed\">"
          @output.puts "  <dl #{indentation_style(number_of_parents)}>"
          @output.puts "  <dt id=\"example_group_#{group_id}\" class=\"passed\">#{h(description)}</dt>"
        end

        def print_example_passed(description, run_time)
          formatted_run_time = "%.5f" % run_time
          @output.puts "    <dd class=\"example passed\">" \
            "<span class=\"passed_spec_name\">#{h(description)}</span>" \
            "<span class='duration'>#{formatted_run_time}s</span></dd>"
        end

        def print_example_failed(pending_fixed, description, run_time, failure_id,
                                 exception, extra_content)
          formatted_run_time = "%.5f" % run_time

          @output.puts "    <dd class=\"example #{pending_fixed ? 'pending_fixed' : 'failed'}\">"
          @output.puts "      <span class=\"failed_spec_name\">#{h(description)}</span>"
          @output.puts "      <span class=\"duration\">#{formatted_run_time}s</span>"
          @output.puts "      <div class=\"failure\" id=\"failure_#{failure_id}\">"
          if exception
            @output.puts "        <div class=\"message\"><pre>#{h(exception[:message])}</pre></div>"
            @output.puts "        <div class=\"backtrace\"><pre>#{h exception[:backtrace]}</pre></div>"
          end
          @output.puts extra_content if extra_content
          @output.puts "      </div>"
          @output.puts "    </dd>"
        end

        def print_example_pending(description, pending_message)
          @output.puts "    <dd class=\"example not_implemented\">" \
            "<span class=\"not_implemented_spec_name\">#{h(description)} " \
            "(PENDING: #{h(pending_message)})</span></dd>"
        end

        def print_summary(duration, example_count, failure_count, pending_count)
          totals = String.new(
            "#{example_count} example#{'s' unless example_count == 1}, "
          )
          totals << "#{failure_count} failure#{'s' unless failure_count == 1}"
          totals << ", #{pending_count} pending" if pending_count > 0

          formatted_duration = "%.5f" % duration

          @output.puts "<script type=\"text/javascript\">" \
            "document.getElementById('duration').innerHTML = \"Finished in " \
            "<strong>#{formatted_duration} seconds</strong>\";</script>"
          @output.puts "<script type=\"text/javascript\">" \
            "document.getElementById('totals').innerHTML = \"#{totals}\";</script>"
          @output.puts "</div>"
          @output.puts "</div>"
          @output.puts "</body>"
          @output.puts "</html>"
        end

        def flush
          @output.flush
        end

        def move_progress(percent_done)
          @output.puts "    <script type=\"text/javascript\">moveProgressBar('#{percent_done}');</script>"
          @output.flush
        end

        def make_header_red
          @output.puts "    <script type=\"text/javascript\">makeRed('rspec-header');</script>"
        end

        def make_header_yellow
          @output.puts "    <script type=\"text/javascript\">makeYellow('rspec-header');</script>"
        end

        def make_example_group_header_red(group_id)
          @output.puts "    <script type=\"text/javascript\">" \
                       "makeRed('div_group_#{group_id}');</script>"
          @output.puts "    <script type=\"text/javascript\">" \
                       "makeRed('example_group_#{group_id}');</script>"
        end

        def make_example_group_header_yellow(group_id)
          @output.puts "    <script type=\"text/javascript\">" \
                       "makeYellow('div_group_#{group_id}');</script>"
          @output.puts "    <script type=\"text/javascript\">" \
                       "makeYellow('example_group_#{group_id}');</script>"
        end

        private

        def indentation_style(number_of_parents)
          "style=\"margin-left: #{(number_of_parents - 1) * 15}px;\""
        end

        REPORT_HEADER = <<-EOF
<div class="rspec-report">

<div id="rspec-header">
  <div id="label">
    <h1>RSpec Code Examples</h1>
  </div>

  <div id="display-filters">
    <input id="passed_checkbox"  name="passed_checkbox"  type="checkbox" checked="checked" onchange="apply_filters()" value="1" /> <label for="passed_checkbox">Passed</label>
    <input id="failed_checkbox"  name="failed_checkbox"  type="checkbox" checked="checked" onchange="apply_filters()" value="2" /> <label for="failed_checkbox">Failed</label>
    <input id="pending_checkbox" name="pending_checkbox" type="checkbox" checked="checked" onchange="apply_filters()" value="3" /> <label for="pending_checkbox">Pending</label>
  </div>

  <div id="summary">
    <p id="totals">&#160;</p>
    <p id="duration">&#160;</p>
  </div>
</div>


<div class="results">
EOF

        GLOBAL_SCRIPTS = <<-EOF

function addClass(element_id, classname) {
  document.getElementById(element_id).className += (" " + classname);
}

function removeClass(element_id, classname) {
  var elem = document.getElementById(element_id);
  var classlist = elem.className.replace(classname,'');
  elem.className = classlist;
}

function moveProgressBar(percentDone) {
  document.getElementById("rspec-header").style.width = percentDone +"%";
}

function makeRed(element_id) {
  removeClass(element_id, 'passed');
  removeClass(element_id, 'not_implemented');
  addClass(element_id,'failed');
}

function makeYellow(element_id) {
  var elem = document.getElementById(element_id);
  if (elem.className.indexOf("failed") == -1) {  // class doesn't includes failed
    if (elem.className.indexOf("not_implemented") == -1) { // class doesn't include not_implemented
      removeClass(element_id, 'passed');
      addClass(element_id,'not_implemented');
    }
  }
}

function apply_filters() {
  var passed_filter = document.getElementById('passed_checkbox').checked;
  var failed_filter = document.getElementById('failed_checkbox').checked;
  var pending_filter = document.getElementById('pending_checkbox').checked;

  assign_display_style("example passed", passed_filter);
  assign_display_style("example failed", failed_filter);
  assign_display_style("example not_implemented", pending_filter);

  assign_display_style_for_group("example_group passed", passed_filter);
  assign_display_style_for_group("example_group not_implemented", pending_filter, pending_filter || passed_filter);
  assign_display_style_for_group("example_group failed", failed_filter, failed_filter || pending_filter || passed_filter);
}

function get_display_style(display_flag) {
  var style_mode = 'none';
  if (display_flag == true) {
    style_mode = 'block';
  }
  return style_mode;
}

function assign_display_style(classname, display_flag) {
  var style_mode = get_display_style(display_flag);
  var elems = document.getElementsByClassName(classname)
  for (var i=0; i<elems.length;i++) {
    elems[i].style.display = style_mode;
  }
}

function assign_display_style_for_group(classname, display_flag, subgroup_flag) {
  var display_style_mode = get_display_style(display_flag);
  var subgroup_style_mode = get_display_style(subgroup_flag);
  var elems = document.getElementsByClassName(classname)
  for (var i=0; i<elems.length;i++) {
    var style_mode = display_style_mode;
    if ((display_flag != subgroup_flag) && (elems[i].getElementsByTagName('dt')[0].innerHTML.indexOf(", ") != -1)) {
      elems[i].style.display = subgroup_style_mode;
    } else {
      elems[i].style.display = display_style_mode;
    }
  }
}
EOF
        # rubocop:enable Layout/LineLength

        GLOBAL_STYLES = <<-EOF
#rspec-header {
  background: #65C400; color: #fff; height: 4em;
}

.rspec-report h1 {
  margin: 0px 10px 0px 10px;
  padding: 10px;
  font-family: "Lucida Grande", Helvetica, sans-serif;
  font-size: 1.8em;
  position: absolute;
}

#label {
  float:left;
}

#display-filters {
  float:left;
  padding: 28px 0 0 40%;
  font-family: "Lucida Grande", Helvetica, sans-serif;
}

#summary {
  float:right;
  padding: 5px 10px;
  font-family: "Lucida Grande", Helvetica, sans-serif;
  text-align: right;
}

#summary p {
  margin: 0 0 0 2px;
}

#summary #totals {
  font-size: 1.2em;
}

.example_group {
  margin: 0 10px 5px;
  background: #fff;
}

dl {
  margin: 0; padding: 0 0 5px;
  font: normal 11px "Lucida Grande", Helvetica, sans-serif;
}

dt {
  padding: 3px;
  background: #65C400;
  color: #fff;
  font-weight: bold;
}

dd {
  margin: 5px 0 5px 5px;
  padding: 3px 3px 3px 18px;
}

dd .duration {
  padding-left: 5px;
  text-align: right;
  right: 0px;
  float:right;
}

dd.example.passed {
  border-left: 5px solid #65C400;
  border-bottom: 1px solid #65C400;
  background: #DBFFB4; color: #3D7700;
}

dd.example.not_implemented {
  border-left: 5px solid #FAF834;
  border-bottom: 1px solid #FAF834;
  background: #FCFB98; color: #131313;
}

dd.example.pending_fixed {
  border-left: 5px solid #0000C2;
  border-bottom: 1px solid #0000C2;
  color: #0000C2; background: #D3FBFF;
}

dd.example.failed {
  border-left: 5px solid #C20000;
  border-bottom: 1px solid #C20000;
  color: #C20000; background: #FFFBD3;
}


dt.not_implemented {
  color: #000000; background: #FAF834;
}

dt.pending_fixed {
  color: #FFFFFF; background: #C40D0D;
}

dt.failed {
  color: #FFFFFF; background: #C40D0D;
}


#rspec-header.not_implemented {
  color: #000000; background: #FAF834;
}

#rspec-header.pending_fixed {
  color: #FFFFFF; background: #C40D0D;
}

#rspec-header.failed {
  color: #FFFFFF; background: #C40D0D;
}


.backtrace {
  color: #000;
  font-size: 12px;
}

a {
  color: #BE5C00;
}

/* Ruby code, style similar to vibrant ink */
.ruby {
  font-size: 12px;
  font-family: monospace;
  color: white;
  background-color: black;
  padding: 0.1em 0 0.2em 0;
}

.ruby .keyword { color: #FF6600; }
.ruby .constant { color: #339999; }
.ruby .attribute { color: white; }
.ruby .global { color: white; }
.ruby .module { color: white; }
.ruby .class { color: white; }
.ruby .string { color: #66FF00; }
.ruby .ident { color: white; }
.ruby .method { color: #FFCC00; }
.ruby .number { color: white; }
.ruby .char { color: white; }
.ruby .comment { color: #9933CC; }
.ruby .symbol { color: white; }
.ruby .regex { color: #44B4CC; }
.ruby .punct { color: white; }
.ruby .escape { color: white; }
.ruby .interp { color: white; }
.ruby .expr { color: white; }

.ruby .offending { background-color: gray; }
.ruby .linenum {
  width: 75px;
  padding: 0.1em 1em 0.2em 0;
  color: #000000;
  background-color: #FFFBD3;
}
EOF

        HTML_HEADER = <<-EOF
<!DOCTYPE html>
<html lang='en'>
<head>
  <title>RSpec results</title>
  <meta http-equiv="Content-Type" content="text/html; charset=utf-8" />
  <meta http-equiv="Expires" content="-1" />
  <meta http-equiv="Pragma" content="no-cache" />
  <style type="text/css">
  body {
    margin: 0;
    padding: 0;
    background: #fff;
    font-size: 80%;
  }
  </style>
  <script type="text/javascript">
    // <![CDATA[
#{GLOBAL_SCRIPTS}
    // ]]>
  </script>
  <style type="text/css">
#{GLOBAL_STYLES}
  </style>
</head>
<body>
EOF
      end
    end
  end
end
