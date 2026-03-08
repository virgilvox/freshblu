import type { Meta, StoryObj } from '@storybook/svelte';
import StatusDot from './StatusDot.svelte';

const meta = {
  title: 'UI/StatusDot',
  component: StatusDot,
  tags: ['autodocs'],
  argTypes: {
    status: { control: 'select', options: ['online', 'warn', 'fault', 'pending'] },
  },
} satisfies Meta<StatusDot>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Online: Story = { args: { status: 'online' } };
export const Warn: Story = { args: { status: 'warn' } };
export const Fault: Story = { args: { status: 'fault' } };
export const Pending: Story = { args: { status: 'pending' } };
