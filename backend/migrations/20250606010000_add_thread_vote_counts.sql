-- スレッドに投票カウントカラムを追加
ALTER TABLE threads ADD COLUMN upvote_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE threads ADD COLUMN downvote_count INTEGER NOT NULL DEFAULT 0;
