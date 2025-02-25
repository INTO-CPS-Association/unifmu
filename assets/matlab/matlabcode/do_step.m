function ok = do_step(current_time, step_size, no_step_prior)
global state_real;
global state_integer;
global state_boolean;
state_real(1, 3) = state_real(1, 1) + state_real(1, 2);
state_integer(1, 3) = state_integer(1, 1) + state_integer(1, 2);
state_boolean(1, 3) = state_boolean(1, 1) | state_boolean(1, 2);
ok = int32(0);
