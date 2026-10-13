# Under `-K`, the literals of a script with no magic comment are in the
# encoding the flag names, and so is __ENCODING__.
p "a".encoding
p __ENCODING__
