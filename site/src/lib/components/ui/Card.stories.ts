import type { Meta, StoryObj } from '@storybook/svelte';
import Card from './Card.svelte';

const meta = {
  title: 'UI/Card',
  component: Card,
  tags: ['autodocs'],
  argTypes: {
    variant: { control: 'select', options: ['default', 'pulse', 'signal', 'fault'] },
  },
} satisfies Meta<Card>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: { variant: 'default', title: 'Device Registry', meta: '12 devices', children: 'Card content goes here.' as any },
};

export const Pulse: Story = {
  args: { variant: 'pulse', title: 'Messages', children: 'Message queue status.' as any },
};

export const Signal: Story = {
  args: { variant: 'signal', title: 'WebSocket', children: 'Connected to gateway pod 1.' as any },
};

export const Fault: Story = {
  args: { variant: 'fault', title: 'Error', children: 'Connection lost.' as any },
};
