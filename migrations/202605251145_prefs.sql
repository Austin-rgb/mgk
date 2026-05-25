CREATE TABLE IF NOT EXISTS preferences(
user TEXT,
address TEXT,
subject TEXT,
UNIQUE(address, subject)
);
 
