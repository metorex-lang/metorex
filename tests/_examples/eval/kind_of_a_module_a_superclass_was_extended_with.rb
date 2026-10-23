module DSL; end
class Base; extend DSL; end
class Sub < Base; end
p Base.kind_of?(DSL), Sub.kind_of?(DSL), Sub.is_a?(DSL), DSL === Sub, Sub.singleton_class.include?(DSL)
p [[//]].include?([//]), [[/a/]].include?([/a/]), [//].include?(//), [[1]].include?([1])
p [[//]].index([//]), [[//]] == [[//]]
p DSL === Base, Comparable === 3, Kernel === Base, Enumerable === [1]
