SELECT COUNT(*) AS profiles, SUM(profile_style IS NOT NULL) AS profiles_with_style FROM social_public_profiles;
