import type { Meta, StoryObj } from '@storybook/svelte';
import Modal from './Modal.svelte';

const meta = {
  title: 'UI/Modal',
  component: Modal,
  tags: ['autodocs'],
} satisfies Meta<Modal>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Open: Story = {
  args: { open: true, title: 'Confirm Unregister', children: 'This action cannot be undone.' as any },
};

export const NoTitle: Story = {
  args: { open: true, children: 'Simple dialog content.' as any },
};
