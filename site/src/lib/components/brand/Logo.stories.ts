import type { Meta, StoryObj } from '@storybook/svelte';
import Logo from './Logo.svelte';

const meta = {
  title: 'Brand/Logo',
  component: Logo,
  tags: ['autodocs'],
  argTypes: {
    size: { control: { type: 'number', min: 16, max: 120 } },
  },
} satisfies Meta<Logo>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { args: { size: 40 } };
export const Large: Story = { args: { size: 80 } };
export const Small: Story = { args: { size: 24 } };
