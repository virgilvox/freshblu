import type { Preview } from '@storybook/svelte';
import '../src/app.css';

const preview: Preview = {
  parameters: {
    backgrounds: {
      default: 'void',
      values: [
        { name: 'void', value: '#060810' },
        { name: 'void-up', value: '#0b1120' },
        { name: 'void-lift', value: '#101828' },
      ],
    },
  },
};

export default preview;
