import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import HomePage from './+page.svelte';
import { tick } from 'svelte';
import * as forge from 'node-forge';
import { goto } from '$app/navigation';
import { applyAction } from '$app/forms';

// Mock the dependencies
vi.mock('$app/navigation', () => ({
  goto: vi.fn()
}));

vi.mock('$app/forms', () => ({
  applyAction: vi.fn()
}));

vi.mock('$lib/utils/rsa-keygen', () => ({
  generateRSAKeyPair: vi.fn(() => Promise.resolve({
    privateKey: 'mock-private-key',
    publicKey: 'mock-public-key'
  }))
}));

describe('HomePage Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the component correctly', async () => {
    render(HomePage);
    
    expect(screen.getByText('Send Messages. Stay Anonymous.')).toBeInTheDocument();
    expect(screen.getByText('Create New Identity')).toBeInTheDocument();
    expect(screen.getByText('Access Existing Identity')).toBeInTheDocument();
  });

  it('shows the modal when create identity button is clicked', async () => {
    render(HomePage);
    
    const createButton = screen.getByText('Create New Identity');
    await fireEvent.click(createButton);
    
    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument();
    });
  });

  it('handles modal cancel correctly', async () => {
    render(HomePage);
    
    const createButton = screen.getByText('Create New Identity');
    await fireEvent.click(createButton);
    
    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument();
    });
    
    const cancelButton = screen.getByText('Cancel');
    await fireEvent.click(cancelButton);
    
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
  });

  describe('handleEnhancedSubmit', () => {
    it('handles successful form submission', async () => {
      render(HomePage);
      
      // Mock the form submission result
      const mockResult = {
        type: 'success',
        data: {
          id: 'ab6548a663c7',
          username: 'cactus'
        }
      };

      let username = $state('');
      let password = $state('');

      // Set up the component state
      const component = render(HomePage, {
        
      });
      
      // Trigger the modal
      const createButton = screen.getByText('Create New Identity');
      await fireEvent.click(createButton);
      await tick();
      
      // Call the submit handler directly
      const submitFn = component.component.handleEnhancedSubmit;
      const result = await submitFn({
        formElement: document.createElement('form'),
        formData: new FormData(),
        action: new URL(''),
        cancel: vi.fn(),
        submitter: document.createElement('form'),
        controller: new AbortController()
      });
      
      // Simulate the result callback
      if (result) {
        await result({ result: mockResult, formData: new FormData() });
      }
      
      // Verify the expected outcomes
      expect(goto).toHaveBeenCalledWith('/dashboard');
      expect(applyAction).toHaveBeenCalledWith(mockResult);
    });

    it('handles error during key generation', async () => {
      // Mock the key generation to fail
      vi.spyOn(require('$lib/utils/rsa-keygen'), 'generateRSAKeyPair')
        .mockRejectedValue(new Error('Key generation failed'));
      
      render(HomePage);
      
      // Set up the component state
      const component = render(HomePage);
      
      // Call the submit handler directly
      const submitFn = component.component.handleEnhancedSubmit;
      const cancelMock = vi.fn();
      const result = await submitFn({
        formElement: document.createElement('form'),
        formData: new FormData(),
        action: new URL(''),
        cancel: cancelMock,
        submitter: document.createElement('form'),
        controller: new AbortController()      
      });
      
      // Verify the error handling
      expect(cancelMock).toHaveBeenCalled();
      expect(component.component.generalError).toEqual('Failed to generate security keys. Please try again.');
    });

    it('handles form validation errors', async () => {
      const mockResult = {
        type: 'failure',
        data: {
          username: 'Username is required',
          password: 'Password must be at least 8 characters'
        }
      };
      
      render(HomePage);
      
      // Set up the component state
      const component = render(HomePage);
      
      // Call the submit handler directly
      const submitFn = component.component.handleEnhancedSubmit;
      const result = await submitFn({
        formElement: document.createElement('form'),
        formData: new FormData(),
        action: new URL(''),
        cancel: vi.fn(),
        submitter: document.createElement('form'),
        controller: new AbortController()            
      });
      
      // Simulate the result callback
      if (result) {
        await result({ result: mockResult, formData: new FormData() });
      }
      
      // Verify the error handling
      expect(component.component.formStatus).toEqual(FormState.VALIDATION_ERROR);
      expect(component.component.formErrors).toEqual(mockResult.data);
      expect(applyAction).toHaveBeenCalledWith(mockResult);
    });
  });

  describe('crypto utility functions', () => {
    it('converts public key to URL-safe Base64 correctly', () => {
      // Mock a public key
      const mockPublicKey = {
        // This is a simplified mock - in a real test you'd use a real key
        n: 'mock-modulus',
        e: 'mock-exponent'
      };
      
      render(HomePage);
      const component = render(HomePage);
      
      // Call the function directly
      const result = component.component.publicKeyToUrlSafeBase64(mockPublicKey);
      
      // Verify the transformation
      expect(result).not.toContain('-----BEGIN PUBLIC KEY-----');
      expect(result).not.toContain('-----END PUBLIC KEY-----');
      expect(result).not.toContain('\n');
      expect(result).not.toContain('+');
      expect(result).not.toContain('/');
    });

    it('converts URL-safe Base64 back to public key correctly', () => {
      const urlSafeBase64 = 'mock-url-safe-base64';
      
      render(HomePage);
      const component = render(HomePage);
      
      // Call the function directly
      const result = component.component.urlSafeBase64ToPublicKey(urlSafeBase64);
      
      // Verify the transformation
      expect(forge.pki.publicKeyFromPem).toHaveBeenCalled();
    });
  });

  it('handles identity confirmation', async () => {
    render(HomePage);
    
    // Set up the component state
    const component = render(HomePage);
    component.component.$set({ 
      username: 'testuser', 
      password: 'testpass',
      showModal: true
    });
    
    await tick();
    
    // Mock the confirm handler
    const confirmSpy = vi.spyOn(component.component, 'handleConfirm');
    
    // Find and click the confirm button
    const confirmButton = screen.getByText('Confirm');
    await fireEvent.click(confirmButton);
    
    // Verify the loading state
    expect(component.component.modalLoading).toBe(true);
    
    // Wait for the timeout to complete
    await waitFor(() => {
      expect(component.component.modalLoading).toBe(false);
      expect(component.component.showModal).toBe(false);
    });
  });
});
