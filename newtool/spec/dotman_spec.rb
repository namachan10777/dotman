require_relative '../sbin/dotman'

describe 'test' do
  it 'expand home' do
    expect(path_expand('$HOME/.cargo')).to eq("#{ENV['HOME']}/.cargo")
  end

  it 'canonicalize' do
    expect(path_expand('$HOME/test/../')).to eq((ENV['HOME']).to_s)
  end
end
