begin; [1][1e30]; rescue => e; p e.message; end
begin; [1][Float::INFINITY]; rescue => e; p e.message; end
begin; [1][-1.5e20]; rescue => e; p e.message; end
