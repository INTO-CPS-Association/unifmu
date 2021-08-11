function [ok, res] = get_string(references)
global state_string;
n = numel(references);
res = strings(1, n);
for i = 1:n
    vr = references(i)-9+1;
    res(i) = state_string(vr);
end
ok = int32(0);