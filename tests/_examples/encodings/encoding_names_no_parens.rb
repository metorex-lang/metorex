# Each encoding answers the names Ruby gives it: its own, then the aliases
# that stand for it. Encoding.find takes any of them, and Encoding.name_list
# lists them in the order Ruby registered them.
# The names that follow the machine's locale are left out, since they name
# whatever encoding the locale does.
LOCALE_NAMES = %w[locale external filesystem]

%w[US-ASCII UTF-8 Shift_JIS ISO-2022-JP UTF-7 GB2312 IBM720 Big5-HKSCS].each do |name|
  encoding = Encoding.find name
  puts "#{encoding.name}: #{(encoding.names - LOCALE_NAMES).inspect}"
end

%w[646 eucJP CP932 ebcdic-cp-us Big5-HKSCS:2008 EUC-CN cp720].each do |alias_name|
  puts "#{alias_name} -> #{Encoding.find(alias_name).name}"
end

p Encoding.name_list.first(12)
p Encoding.name_list.last(5)
p Encoding.list.size
p Encoding.aliases["646"]
p Encoding::CP720
p "text".force_encoding("eucJP").encoding
