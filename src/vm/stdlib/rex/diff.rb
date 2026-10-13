module Rex
  # The line diff failure messages end with, written as diff-lcs writes one:
  # the pairing of lines comes from `__matches__`, which Metorex computes in
  # Rust, and the hunks and their unified form follow diff-lcs.
  module Diff
    Change = Struct.new(:action, :position, :element) do
      def adding?
        action == "+"
      end

      def deleting?
        action == "-"
      end
    end

    # The runs of changes between `old` and `new`, each an Array of
    # Changes, with the lines the two share left out.
    def self.diff(old, new)
      matches = __matches__(old, new)
      pieces = []
      piece = []
      finish_piece = lambda do
        pieces << piece unless piece.empty?
        piece = []
      end
      old_at = 0
      new_at = 0
      matches.each do |paired|
        if paired.nil?
          piece << Change.new("-", old_at, old[old_at]) unless old[old_at].nil?
        else
          while new_at < paired
            piece << Change.new("+", new_at, new[new_at])
            new_at += 1
          end
          finish_piece.call
          new_at += 1
        end
        old_at += 1
      end
      while old_at < old.size || new_at < new.size
        if old_at == old.size && new_at < new.size
          loop do
            piece << Change.new("+", new_at, new[new_at])
            new_at += 1
            break unless new_at < new.size
          end
        end
        if new_at == new.size && old_at < old.size
          loop do
            piece << Change.new("-", old_at, old[old_at])
            old_at += 1
            break unless new_at < new.size
          end
        end
        if old_at < old.size
          piece << Change.new("-", old_at, old[old_at])
          old_at += 1
        end
        if new_at < new.size
          piece << Change.new("+", new_at, new[new_at])
          new_at += 1
        end
      end
      finish_piece.call
      pieces
    end

    class Block
      attr_reader :changes, :insert, :remove

      def initialize(piece)
        @changes = piece
        @remove = piece.select(&:deleting?)
        @insert = piece.select(&:adding?)
      end

      def diff_size
        @insert.size - @remove.size
      end
    end

    # One run of changes with the lines of context around it, which merges
    # with the hunk before it when their context overlaps.
    class Hunk
      attr_reader :blocks, :start_old, :start_new, :end_old, :end_new, :file_length_difference

      def initialize(data_old, data_new, piece, context, file_length_difference)
        @blocks = [Block.new(piece)]
        @data_old = data_old
        @data_new = data_new
        @old_empty = data_old.empty? || (data_old.size == 1 && data_old[0].empty?)
        @new_empty = data_new.empty? || (data_new.size == 1 && data_new[0].empty?)

        before = after = file_length_difference
        after += @blocks[0].diff_size
        @file_length_difference = after

        removed = @blocks[0].remove
        inserted = @blocks[0].insert
        first_old = removed.empty? ? nil : removed[0].position
        last_old = removed.empty? ? nil : removed[-1].position
        first_new = inserted.empty? ? nil : inserted[0].position
        last_new = inserted.empty? ? nil : inserted[-1].position

        @start_old = first_old || (first_new - before)
        @start_new = first_new || (first_old + before)
        @end_old = last_old || (last_new - after)
        @end_new = last_new || (last_old + after)
        add_context(context)
      end

      def merge(hunk)
        return unless overlaps?(hunk)

        @start_old = hunk.start_old
        @start_new = hunk.start_new
        blocks.unshift(*hunk.blocks)
      end

      def overlaps?(hunk)
        hunk && ((@start_old - hunk.end_old) <= 1 || (@start_new - hunk.end_new) <= 1)
      end

      def unified_diff
        text = +"@@ -#{unified_range(:old)} +#{unified_range(:new)} @@\n"
        low = @start_old
        added = 0
        removed = 0
        lines = @data_old[low..@end_old].map { |line| +" #{line.chomp}" }
        @blocks.each do |block|
          block.remove.each do |change|
            lines[change.position - low + added][0, 1] = "-"
            removed += 1
          end
          block.insert.each do |change|
            lines[change.position - @start_new + removed, 0] = "+#{@data_new[change.position].chomp}"
            added += 1
          end
        end
        text << lines.join("\n")
      end

      private

      def add_context(context)
        return if context.nil? || context.zero?

        add_start = context > @start_old ? @start_old : context
        @start_old -= add_start
        @start_new -= add_start
        add_end = @end_old + context >= @data_old.size ? @data_old.size - @end_old - 1 : context
        @end_old += add_end
        @end_new += add_end
      end

      def unified_range(side)
        if side == :old
          return "0,0" if @old_empty

          first = @start_old + 1
          last = @end_old + 1
        else
          return "0,0" if @new_empty

          first = @start_new + 1
          last = @end_new + 1
        end
        length = last - first + 1
        length <= 1 ? last.to_s : "#{first},#{length}"
      end
    end
  end
end
