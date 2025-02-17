function [ok, res] = get_integer(references)
global state_integer;
n = numel(references);
res = zeros(1, n, 'int32');
for i = 1:n
    vr = references(i)-3+1;
    res(i) = state_integer(vr);
end
ok = int32(0);