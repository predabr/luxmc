let feature = $state('');
export const planPrompt = {
    get feature() { return feature; },
    open(value: string) { feature = value; },
    close() { feature = ''; }
};
