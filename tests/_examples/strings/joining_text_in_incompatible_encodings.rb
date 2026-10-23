bin = [0xff].pack("C")
begin; p("ё" + bin); rescue => e; p e.message; end
begin; p(bin + "ё"); rescue => e; p e.message; end
begin; p("ё".dup << bin); rescue => e; p e.message; end
begin; p("#{"ё"}#{bin}"); rescue => e; p e.message; end
begin; p(["ё", bin].join); rescue => e; p e.message; end
