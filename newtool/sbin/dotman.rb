#!/usr/bin/ruby

module InstallType
  RUN_SHELL_WITH_COND = 0
end

# インストール単位
class InstallUnit
  def initialize(type)
    @type = type
  end

  def execute
    puts 'Please implements this'
  end
end

# インストールスクリプトの実行
class RunShellWithCond < InstallUnit
  def initialize(cmd, cond)
    super(InstallType::RUN_SHELL_WITH_COND)
    @cmd = cmd
    @cond = cond
  end

  def execute
    system(@cmd) if @cond.call
  end
end

def path_expand(path)
  File.expand_path(`echo -n #{path}`)
end

def test(path)
  # TODO: エラー処理
  File.exist?(path_expand(path))
end

full = [
  RunShellWithCond.new(
    "sh -c \"curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\"",
    -> { !test('$HOME/.cargo/bin/rustup') }
  )
]

# test
full[0].execute
