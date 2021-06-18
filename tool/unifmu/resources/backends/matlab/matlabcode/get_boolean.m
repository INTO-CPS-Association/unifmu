function [ok, res] = get_boolean(references)
global state_boolean;
n = numel(references);
res = zeros(1, n, 'logical');
for i = 1:n
    vr = references(i)-6+1;
    res(i) = state_boolean(vr);
end
ok = int32(0);
