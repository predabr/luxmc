SELECT id, enabled, bank, updated_at, CASE WHEN length(pix_key) BETWEEN 5 AND 100 AND pix_key NOT GLOB '*[^a-zA-Z0-9@.+_-]*' THEN 1 ELSE 0 END AS pix_configured, owner_password_hash IS NOT NULL AS panel_password_configured FROM lux_commerce_config WHERE id = 1;
SELECT COUNT(*) AS active_owner FROM social_users WHERE id = '43d26fda-6fbd-4727-8680-f96aca47dcf3' AND removed_at = 0;
