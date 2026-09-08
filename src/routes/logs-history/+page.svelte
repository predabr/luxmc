<script lang="ts">
	import Heading from "$lib/components/ui/Heading.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import LogsViewer from "$lib/components/ui/LogsViewer.svelte";
	import { History } from "lucide-svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();
</script>

<div class="mx-auto flex h-full max-w-6xl flex-col gap-4">
	<header class="flex items-center gap-2">
		<History class="h-5 w-5" style="color: rgb(45, 212, 191);" />
		<Heading>{t("logs.historyTitle")}</Heading>
	</header>
	<p class="text-xs" style="color: rgb(var(--fg-muted));">
		{t("logs.historyDesc")}
	</p>
	<Card>
		<LogsViewer />
	</Card>
</div>
