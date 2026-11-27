-- Whose turn it is: a claim takes users longest since their last claim first, never claimed before any,
-- and stamps every user it took a job from, so users with due jobs take turns across passes
ALTER TABLE users ADD COLUMN claimed_at TIMESTAMPTZ;
