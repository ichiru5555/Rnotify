CREATE TABLE `earthquake` (
  `id` int(11) NOT NULL,
  `earthquake_id` text NOT NULL,
  `type` text NOT NULL,
  `time` datetime NOT NULL,
  `magnitude` text NOT NULL,
  `depth` text NOT NULL,
  `intensity` text NOT NULL,
  `location` text NOT NULL,
  `tsunami` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

ALTER TABLE `earthquake`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `earthquake`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;
COMMIT;
