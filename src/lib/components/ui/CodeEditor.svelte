<script lang="ts">
	import { onMount } from "svelte";
	import { EditorView, basicSetup } from "codemirror";
	import { EditorState } from "@codemirror/state";

	type Props = {
		value: string;
		readOnly?: boolean;
		onChange?: (val: string) => void;
	};

	let { value = $bindable(), readOnly = false, onChange }: Props = $props();

	let container = $state<HTMLDivElement | null>(null);
	let view: EditorView | null = null;
	let isUpdatingFromProp = false;

	$effect(() => {
		if (!container) return;

		const updateListener = EditorView.updateListener.of((update) => {
			if (update.docChanged && !isUpdatingFromProp) {
				const newVal = update.state.doc.toString();
				value = newVal;
				onChange?.(newVal);
			}
		});

		const theme = EditorView.theme({
			"&": {
				backgroundColor: "rgb(var(--bg-elevated))",
				color: "rgb(var(--fg))",
				height: "100%",
				fontSize: "12px",
				fontFamily: "monospace"
			},
			".cm-content": {
				caretColor: "rgb(var(--brand-400))"
			},
			"&.cm-focused .cm-cursor": {
				borderLeftColor: "rgb(var(--brand-400))"
			},
			"&.cm-focused .cm-selectionBackground, ::selection": {
				backgroundColor: "rgb(var(--brand-500) / 0.25)"
			},
			".cm-gutters": {
				backgroundColor: "rgb(var(--bg))",
				color: "rgb(var(--fg-subtle))",
				borderRight: "1px solid rgb(var(--fg) / 0.05)"
			}
		});

		const state = EditorState.create({
			doc: value,
			extensions: [
				basicSetup,
				updateListener,
				theme,
				EditorState.readOnly.of(readOnly)
			]
		});

		view = new EditorView({
			state,
			parent: container
		});

		return () => {
			if (view) {
				view.destroy();
				view = null;
			}
		};
	});

	$effect(() => {
		if (view && value !== view.state.doc.toString()) {
			isUpdatingFromProp = true;
			view.dispatch({
				changes: { from: 0, to: view.state.doc.length, insert: value }
			});
			isUpdatingFromProp = false;
		}
	});
</script>

<div 
	bind:this={container} 
	class="w-full h-full min-h-[280px] rounded-2xl overflow-hidden border border-fg/10"
></div>

<style>
	:global(.cm-editor) {
		height: 100% !important;
	}
	:global(.cm-scroller) {
		overflow: auto !important;
	}
</style>
