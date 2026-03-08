import type { Meta, StoryObj } from '@storybook/svelte';
import Badge from './Badge.svelte';

const meta = {
  title: 'UI/Badge',
  component: Badge,
  tags: ['autodocs'],
  argTypes: {
    variant: { control: 'select', options: ['online', 'warn', 'fault', 'pending', 'pulse', 'muted'] },
  },
} satisfies Meta<Badge>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Online: Story = { args: { variant: 'online', children: 'Online' as any } };
export const Warn: Story = { args: { variant: 'warn', children: 'Warning' as any } };
export const Fault: Story = { args: { variant: 'fault', children: 'Fault' as any } };
export const Pending: Story = { args: { variant: 'pending', children: 'Pending' as any } };
export const Pulse: Story = { args: { variant: 'pulse', children: 'HTTP' as any } };
export const Muted: Story = { args: { variant: 'muted', children: 'Offline' as any } };
