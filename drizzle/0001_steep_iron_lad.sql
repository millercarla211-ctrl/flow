CREATE TABLE `friday_workspace_snapshot` (
	`userId` text PRIMARY KEY NOT NULL,
	`version` integer DEFAULT 1 NOT NULL,
	`payload` text NOT NULL,
	`createdAt` integer NOT NULL,
	`updatedAt` integer NOT NULL,
	FOREIGN KEY (`userId`) REFERENCES `user`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE INDEX `friday_workspace_snapshot_updated_at_idx` ON `friday_workspace_snapshot` (`updatedAt`);