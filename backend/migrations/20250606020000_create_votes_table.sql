-- 投票テーブルの追加
CREATE TABLE votes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    thread_id UUID NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
    vote_type VARCHAR(10) NOT NULL CHECK (vote_type IN ('upvote', 'downvote')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_thread_unique UNIQUE(user_id, thread_id)
);

-- 投票時にスレッドのカウントを更新するトリガー
CREATE OR REPLACE FUNCTION update_thread_vote_counts() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.vote_type = 'upvote' THEN
            UPDATE threads SET upvote_count = upvote_count + 1 WHERE id = NEW.thread_id;
        ELSE
            UPDATE threads SET downvote_count = downvote_count + 1 WHERE id = NEW.thread_id;
        END IF;
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.vote_type = 'upvote' AND NEW.vote_type = 'downvote' THEN
            UPDATE threads SET upvote_count = upvote_count - 1, downvote_count = downvote_count + 1 WHERE id = NEW.thread_id;
        ELSIF OLD.vote_type = 'downvote' AND NEW.vote_type = 'upvote' THEN
            UPDATE threads SET downvote_count = downvote_count - 1, upvote_count = upvote_count + 1 WHERE id = NEW.thread_id;
        END IF;
    ELSIF TG_OP = 'DELETE' THEN
        IF OLD.vote_type = 'upvote' THEN
            UPDATE threads SET upvote_count = upvote_count - 1 WHERE id = OLD.thread_id;
        ELSE
            UPDATE threads SET downvote_count = downvote_count - 1 WHERE id = OLD.thread_id;
        END IF;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER votes_update_thread_counts
AFTER INSERT OR UPDATE OR DELETE ON votes
FOR EACH ROW EXECUTE FUNCTION update_thread_vote_counts();
