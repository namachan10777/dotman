require_relative '../dotman'

describe 'replace_env_vars' do
  it 'replace $HOME' do
    expect(replace_env_vars('$HOME', { 'HOME' => 'home' })).to eq('home')
  end
end

describe 'test' do
  it 'expand home' do
    expect(path_expand('$HOME/.cargo')).to eq("#{ENV['HOME']}/.cargo")
  end

  it 'canonicalize' do
    expect(path_expand('$HOME/test/../')).to eq((ENV['HOME']).to_s)
  end
end
