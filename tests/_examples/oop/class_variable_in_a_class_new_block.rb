class Mailer
  def build(settings)
    Class.new do
      @@settings = settings
      def settings
        @@settings
      end
    end
  end
end

made = Mailer.new.build({host: "mail.example.com"})
p made.new.settings
p Mailer.class_variable_get(:@@settings)
p made.class_variables
p Mailer.class_variables
