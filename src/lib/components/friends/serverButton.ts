import { button as base, type ButtonVariant, type ButtonSize } from '$lib/components/ui/button';

export function button(options: { variant?: ButtonVariant; size?: ButtonSize; class?: string; block?: boolean } = {}) {
    return `${base(options)} server-action`;
}
