import type { Meta, StoryObj } from '@storybook/svelte';
import Button from './Button.svelte';

const meta = {
  title: 'UI/Button',
  component: Button,
  tags: ['autodocs'],
  argTypes: {
    variant: { control: 'select', options: ['primary', 'ghost', 'muted', 'signal'] },
    size: { control: 'select', options: ['sm', 'md', 'lg'] },
  },
} satisfies Meta<Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: { variant: 'primary', size: 'md', children: 'Register Device' as any },
};

export const Ghost: Story = {
  args: { variant: 'ghost', size: 'md', children: 'View Docs' as any },
};

export const Muted: Story = {
  args: { variant: 'muted', size: 'md', children: 'Cancel' as any },
};

export const Signal: Story = {
  args: { variant: 'signal', size: 'md', children: 'Connect' as any },
};

export const Small: Story = {
  args: { variant: 'primary', size: 'sm', children: 'Small' as any },
};

export const Large: Story = {
  args: { variant: 'primary', size: 'lg', children: 'Large Button' as any },
};
