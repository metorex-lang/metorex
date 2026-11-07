text = "begin 644 Happy.txt\nM2&%P<'D@=&]D87D@9F]R('1O(&)E(&]N92!O9B!P96%C92!A;F0@<V5R96YE\n'('1I;64N\"@``\n`\nend\n"
p text.sub(/\Abegin \d+ [^\n]*\n/, "").unpack1("u")
p ["Happy today"].pack("u"), ["Happy ああr"].pack("u")
p "-2&%P<'D@XX&\"XX&\"<@``\n".unpack1("u"), "+2&%P<'D@=&]D87D`\n".unpack1("u")
p ["x" * 50].pack("u").lines.size, [""].pack("u")
