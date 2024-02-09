CREATE TABLE word (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
    `value` VARCHAR(100) NOT NULL,
    `lang` VARCHAR(3) NOT NULL,
    PRIMARY KEY (`id`)
);

CREATE TABLE variant (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
    `fk_word_id` int unsigned NOT NULL,
    `name` VARCHAR(100) NOT NULL,
    `value` VARCHAR(100) NOT NULL,
    PRIMARY KEY (`id`),
    CONSTRAINT `variant_word_ibfk_1` FOREIGN KEY (`fk_word_id`) REFERENCES `word` (`id`)
);

CREATE TABLE translation (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
    `fk_word_1_id` int unsigned NOT NULL,
    `fk_word_2_id` int unsigned NOT NULL,
    PRIMARY KEY (`id`),
    CONSTRAINT `variant_word1_ibfk_1` FOREIGN KEY (`fk_word_1_id`) REFERENCES `word` (`id`),
    CONSTRAINT `variant_word2_ibfk_1` FOREIGN KEY (`fk_word_2_id`) REFERENCES `word` (`id`)
);
