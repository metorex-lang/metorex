Order = Struct.new(:id, :total, :items) do
  def paid? = total.positive?
  def has_items? = !items.empty?
  def ship(carrier, speed: :ground, signature:) = [carrier, speed, signature]
  def tag(*labels) = labels
end

describe "matchers that pass" do
  let(:order) { Order.new(7, 4200, %w[book lamp]) }

  it("eq, eql, equal, be") do
    expect(order.id).to eq(7)
    expect(1.0).to eql(1.0)
    expect(:symbol).to equal(:symbol)
    expect(order).to be
    expect(nil).not_to be
    expect(true).to be(true)
  end

  it("truthiness") do
    expect(order.paid?).to be_truthy
    expect(nil).to be_falsey
    expect(nil).to be_nil
  end

  it("comparisons") do
    expect(order.total).to be > 4000
    expect(order.total).to be_between(4000, 4200)
    expect(4.2).to be_within(0.05).of(4.19)
    expect(110).to be_within(10).percent_of(100)
    expect(order.total).not_to be < 1
  end

  it("types") do
    expect(order).to be_a(Order)
    expect(order).to be_an_instance_of(Order)
    expect(3).to be_kind_of(Numeric)
    expect(order).to respond_to(:total, :items)
    expect("text").to respond_to(:upcase).with(1).argument
    expect(order).not_to respond_to(:refund)
    expect(order).to respond_to(:ship).with(1).argument.and_keywords(:speed, :signature)
    expect(order).not_to respond_to(:ship).with_keywords(:speed)
    expect(order).to respond_to(:tag).with_unlimited_arguments
  end

  it("predicates") do
    expect(order).to be_paid
    expect(order).to have_items
    expect([]).to be_empty
    expect({ a: 1 }).to have_key(:a)
  end

  it("collections") do
    expect(order.items).to include("book")
    expect(order.items).to contain_exactly("lamp", "book")
    expect(order.items).to match_array(%w[book lamp])
    expect(order.items).to start_with("book")
    expect(order.items).to end_with("lamp")
    expect([1, 3, 5]).to all(be_odd)
    expect({ id: 7, items: ["book"] }).to include(id: 7)
    expect(1..10).to cover(3, 9)
  end

  it("match") do
    expect("order 7").to match(/order \d/)
    expect(id: 7, items: %w[book]).to match(id: Integer, items: [a_string_starting_with("bo")])
    expect(order.to_h).to have_attributes(size: 3)
    expect(order).to have_attributes(id: 7, total: a_value > 4000)
  end

  it("blocks") do
    expect { raise ArgumentError, "bad total" }.to raise_error(ArgumentError, "bad total")
    expect { raise ArgumentError, "bad total" }.to raise_error(/total/)
    expect { raise ArgumentError }.to raise_error { |error| expect(error).to be_a(ArgumentError) }
    expect { 1 }.not_to raise_error
    expect { throw :done, 3 }.to throw_symbol(:done, 3)
    items = []
    expect { items << 1 }.to change { items.size }.from(0).to(1)
    expect { items << 2 }.to change(items, :size).by(1)
    expect { items.first }.not_to change { items.size }
    expect { print "ok" }.to output("ok").to_stdout
    expect { warn "careful" }.to output(/careful/).to_stderr
    expect { |probe| [1, 2].each(&probe) }.to yield_control.twice
    expect { |probe| [[1, 2]].each(&probe) }.to yield_with_args([1, 2])
    expect { |probe| [1, 2].each(&probe) }.to yield_successive_args(1, 2)
    expect { |probe| 1.then { probe.to_proc.call } }.to yield_with_no_args
  end

  it("composition") do
    expect(order.total).to be_positive.and be_even
    expect(order.id).to eq(8).or eq(7)
    expect(order.items).to include(a_string_matching(/la/))
    expect(order.total).to satisfy { |total| total % 100 == 0 }
  end

  it("aggregates failures that do not happen") do
    aggregate_failures do
      expect(order.id).to eq(7)
      expect(order.items.size).to eq(2)
    end
  end
end
