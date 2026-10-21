message = "file.rb:130: warning: Pass a type or block to #responses, set config.responses_without_block to :silence_deprecation_warning, or use #extract_responses or #clear_responses"
pattern = /
        (?=(?-x)Pass a type or block to #responses\b)
        (?=.*config\.responses_without_block.*:silence_deprecation_warning\b)
        (?=.*\#extract_responses\b)
           .*\#clear_responses\b
      /ix
p message.match?(pattern)
p "a #b".match?(/(?-x)a #b/x), "ab".match?(/a #b/x), "a #b".match?(/x(?-x: #b)|a(?-x) #b/x)
p "a".match?(/(?=(?-x)a) a/x), "ab".match?(/(?:(?-x)a) b/x), "AB".match?(/(?:(?-i)A)b/i)
