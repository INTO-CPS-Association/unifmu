function [ok, res] = get_real(references)
global state_real;
n = numel(references);
res = zeros(1, n);
for i = 1:n
    vr = references(i)+1;
    res(i) = state_real(vr);
end
ok = int32(0);