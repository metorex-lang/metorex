# A numbered parameter read only inside a regexp's interpolation is still
# the block's parameter, as net-imap's tests build a pattern from each name.
names = %w[debug sasl_ir]
p names.map { / #{_1}=.+/.source }
p names.map { "#{it}!" }
