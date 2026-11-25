require "cgi/escape"

p CGI.class
p CGI.singleton_class.include?(CGI::Escape)
p CGI.unescape_html("&lt;b&gt; &amp; &quot;")
p CGI.escape_html("<a href='x'>")
p CGI.h("&")
p CGI.escape_uri_component("a b/c")
p CGI.unescape_uri_component("a%20b")
p CGI.escape_element("<br><a>", "a")
p CGI.unescape_element("&lt;br&gt;&lt;a&gt;", "a")

class Page
  include CGI::Escape

  def show(text) = escape_html(text)
end
p Page.new.show("<p>")
