import type { Meta, StoryObj } from '@storybook/svelte';
import LogoFull from './LogoFull.svelte';

const meta = {
  title: 'Brand/LogoFull',
  component: LogoFull,
  tags: ['autodocs'],
} satisfies Meta<LogoFull>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
