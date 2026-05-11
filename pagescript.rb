class Pagescript < Formula
  desc "Compile .page files to standalone HTML"
  homepage "https://github.com/oliver-morrow/pagescript"
  head "https://github.com/oliver-morrow/pagescript.git", branch: "main"

  depends_on "rust" => :build

  def install
    # Navigate to the rust package directory
    Dir.chdir("rust/pagescript-rs") do
      system "cargo", "install", *std_cargo_args
    end
  end

  test do
    # Create a minimal valid page
    (testpath/"test.page").write <<~EOS
      ::page id=test title="Test"
        ::text value="Hello Brew"
        ::/text
      ::/page
    EOS

    # Run validation
    system bin/"pagescript-rs", "validate", "test.page"
    
    # Run rendering and check output
    output = shell_output("#{bin}/pagescript-rs render test.page")
    assert_match "Hello Brew", output
  end
end
