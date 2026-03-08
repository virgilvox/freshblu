import type { Meta, StoryObj } from '@storybook/svelte';
import Input from './Input.svelte';

const meta = {
  title: 'UI/Input',
  component: Input,
  tags: ['autodocs'],
} satisfies Meta<Input>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: { label: 'Device UUID', placeholder: 'Enter UUID...', note: 'The target device identifier.' },
};

export const NoLabel: Story = {
  args: { placeholder: 'Search devices...' },
};

export const WithValue: Story = {
  args: { label: 'Token', value: 'abc123def456', type: 'password' },
};
