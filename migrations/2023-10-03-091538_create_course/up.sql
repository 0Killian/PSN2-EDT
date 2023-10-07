-- Your SQL goes here
CREATE TABLE IF NOT EXISTS course (
    id INT NOT NULL AUTO_INCREMENT,
    date DATE NOT NULL,
    category ENUM('dev', 'infra', 'devinfra', 'marketing', 'common') NOT NULL,
    start TIME NOT NULL,
    end TIME NOT NULL,
    subject VARCHAR(255) NOT NULL,
    teacher VARCHAR(255) NOT NULL,
    classroom VARCHAR(255) NOT NULL,
    remote BOOLEAN NOT NULL,
    bts BOOLEAN NOT NULL,
    PRIMARY KEY (id)
);