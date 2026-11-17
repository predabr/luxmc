export const catalogTypeState = $state<{ value: string }>({ value: "Modpack" });

export function setCatalogType(value: string) {
	catalogTypeState.value = value;
}

export function catalogTypeQuery(value: string) {
	return value.toLowerCase().replace(/\s+/g, "");
}
