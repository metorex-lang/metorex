# Opening shared libraries and finding the symbols in them.
module Fiddle
  class Error < StandardError; end
  class DLError < Error; end

  RTLD_GLOBAL, RTLD_LAZY, RTLD_NOW = __dynamic_library__(:flags)

  TYPE_VOID = 0
  TYPE_VOIDP = 1
  TYPE_CHAR = 2
  TYPE_SHORT = 3
  TYPE_INT = 4
  TYPE_LONG = 5
  TYPE_LONG_LONG = 6
  TYPE_FLOAT = 7
  TYPE_DOUBLE = 8
  TYPE_VARIADIC = 9
  TYPE_CONST_STRING = 10
  TYPE_BOOL = 11
  TYPE_UCHAR = -TYPE_CHAR
  TYPE_USHORT = -TYPE_SHORT
  TYPE_UINT = -TYPE_INT
  TYPE_ULONG = -TYPE_LONG
  TYPE_ULONG_LONG = -TYPE_LONG_LONG
  TYPE_INT8_T = TYPE_CHAR
  TYPE_UINT8_T = TYPE_UCHAR
  TYPE_INT16_T = TYPE_SHORT
  TYPE_UINT16_T = TYPE_USHORT
  TYPE_INT32_T = TYPE_INT
  TYPE_UINT32_T = TYPE_UINT
  TYPE_INT64_T = TYPE_LONG
  TYPE_UINT64_T = TYPE_ULONG
  TYPE_SIZE_T = TYPE_ULONG
  TYPE_SSIZE_T = TYPE_LONG
  TYPE_PTRDIFF_T = TYPE_LONG
  TYPE_INTPTR_T = TYPE_LONG
  TYPE_UINTPTR_T = TYPE_ULONG

  # Metorex runs on 64-bit Linux and macOS, where long and pointers are
  # 8 bytes.
  { BOOL: 1, CHAR: 1, UCHAR: 1, SHORT: 2, USHORT: 2, INT: 4, UINT: 4, LONG: 8, ULONG: 8,
    LONG_LONG: 8, ULONG_LONG: 8, FLOAT: 4, DOUBLE: 8, VOIDP: 8, CONST_STRING: 8,
    INT8_T: 1, UINT8_T: 1, INT16_T: 2, UINT16_T: 2, INT32_T: 4, UINT32_T: 4,
    INT64_T: 8, UINT64_T: 8, SIZE_T: 8, SSIZE_T: 8, PTRDIFF_T: 8, INTPTR_T: 8,
    UINTPTR_T: 8 }.each do |type, size|
    const_set(:"SIZEOF_#{type}", size)
    const_set(:"ALIGN_#{type}", size) unless (type.start_with?("U") && type != :UINTPTR_T) || type == :CONST_STRING
  end

  # An address in the program's memory, read a byte or a run of bytes at a
  # time.
  class Pointer
    def self.[](address)
      new address
    end

    def initialize(address, size = 0, free = nil)
      @address = address.to_i
      @size = size
    end

    def to_i
      @address
    end
    alias to_int to_i

    attr_reader :size

    # The C string the pointer names, or the `length` bytes from it.
    def to_s(length = nil)
      return Fiddle.__dynamic_library__(:read, @address, length) unless length.nil?
      Fiddle.__dynamic_library__(:string, @address)
    end
    alias to_str to_s

    def null?
      @address.zero?
    end

    # The signed byte at `offset`, or the `length` bytes from it.
    def [](offset, length = nil)
      return Fiddle.__dynamic_library__(:read, @address + offset, length) unless length.nil?
      Fiddle.__dynamic_library__(:read, @address + offset, 1).unpack1("c")
    end
  end

  # A shared library the program has opened, or the program itself.
  class Handle
    RTLD_GLOBAL = Fiddle::RTLD_GLOBAL
    RTLD_LAZY = Fiddle::RTLD_LAZY
    RTLD_NOW = Fiddle::RTLD_NOW

    def self.sym(name)
      DEFAULT.sym(name)
    end

    def self.[](name)
      sym(name)
    end

    def initialize(library = nil, flags = RTLD_LAZY | RTLD_GLOBAL)
      @address = Fiddle.__dynamic_library__(:open, library, flags)
      @closed = false
      @closable = false
    end

    def sym(name)
      raise DLError, "closed handle" if @closed
      unless name.is_a?(String)
        unless name.respond_to?(:to_str)
          raise TypeError, "no implicit conversion of #{name.class} into String"
        end
        name = name.to_str
      end
      Fiddle.__dynamic_library__(:symbol, @address, name)
    end
    alias [] sym

    def close
      raise DLError, "dlclose() called too many times" if @closed
      @closed = true
      Fiddle.__dynamic_library__(:close, @address)
    end

    def to_i
      @address
    end

    def to_ptr
      @address
    end

    def disable_close
      @closable = false
    end

    def enable_close
      @closable = true
    end

    def close_enabled?
      @closable
    end

    def self.__default__
      made = allocate
      made.instance_variable_set(:@address, Fiddle.__dynamic_library__(:default))
      made.instance_variable_set(:@closed, false)
      made.instance_variable_set(:@closable, false)
      made
    end

    DEFAULT = __default__
  end

  # A C function at an address, called with the argument types it was
  # declared with and answering its declared return type.
  class Function
    DEFAULT = 1

    attr_reader :abi, :name, :ptr

    def initialize(ptr, args, ret_type, abi = DEFAULT, name: nil, need_gvl: false)
      @ptr = ptr.to_i
      @args = args.map { |type| Integer(type) }
      @ret_type = Integer(ret_type)
      @abi = abi
      @name = name
      @need_gvl = need_gvl
    end

    def call(*arguments)
      unless arguments.size == @args.size
        raise ArgumentError, "wrong number of arguments (given #{arguments.size}, expected #{@args.size})"
      end
      passed = arguments.zip(@args).map do |argument, type|
        next argument unless type == TYPE_VOIDP || type == TYPE_CONST_STRING
        case argument
        when nil, Integer, String then argument
        when Pointer then argument.to_i
        else argument.respond_to?(:to_ptr) ? argument.to_ptr.to_i : argument
        end
      end
      answered = Fiddle.__dynamic_library__(:call, @ptr, @args, @ret_type, passed)
      @ret_type == TYPE_VOIDP ? Pointer.new(answered) : answered
    end

    def need_gvl?
      @need_gvl
    end

    def to_i
      @ptr
    end

    def to_proc
      method(:call).to_proc
    end
  end

  def self.dlopen(library)
    Handle.new(library)
  end
end
