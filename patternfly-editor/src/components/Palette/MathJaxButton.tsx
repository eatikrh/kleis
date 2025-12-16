/**
 * MathJax-aware button component
 * 
 * Renders LaTeX strings as SVG using MathJax, matching static/index.html behavior
 */

import React, { useEffect, useRef } from 'react';
import { Button } from '@patternfly/react-core';
import type { ButtonConfig } from './buttonConfigs';
import type { EditorNode } from '../../types/ast';
import { cloneAndRenumber } from '../../types/ast';
import { getTemplate } from './astTemplates';

interface MathJaxButtonProps {
  config: ButtonConfig;
  onInsert: (ast: EditorNode) => void;
}

declare global {
  interface Window {
    MathJax?: {
      typesetPromise: (elements: Element[]) => Promise<void>;
      startup?: {
        ready: () => void;
        promise: Promise<void>;
      };
    };
  }
}

export const MathJaxButton = React.memo(function MathJaxButton({ config, onInsert }: MathJaxButtonProps) {
  const buttonRef = useRef<HTMLButtonElement>(null);
  const contentRef = useRef<HTMLSpanElement>(null);

  const handleClick = (e: React.MouseEvent) => {
    // Stop propagation to prevent document-level handlers from clearing focus
    e.stopPropagation();
    
    // If custom action is provided, use it instead of inserting template
    if (config.customAction) {
      config.customAction();
      return;
    }
    
    const template = getTemplate(config.template);
    if (template) {
      const ast = cloneAndRenumber(template);
      onInsert(ast);
    } else {
      console.warn(`No template found for: ${config.template}`);
    }
  };

  // Render MathJax when component mounts or LaTeX changes
  useEffect(() => {
    if (!config.latex || !contentRef.current) return;

    let isMounted = true;
    let checkInterval: ReturnType<typeof setInterval> | null = null;
    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    const renderMathJax = async () => {
      if (!isMounted) return;
      
      // Wait for MathJax to load
      if (!window.MathJax) {
        // Check if MathJax script is loading
        checkInterval = setInterval(() => {
          if (window.MathJax && isMounted) {
            if (checkInterval) clearInterval(checkInterval);
            checkInterval = null;
            renderMathJax();
          }
        }, 100);
        
        // Timeout after 5 seconds
        timeoutId = setTimeout(() => {
          if (checkInterval) clearInterval(checkInterval);
          checkInterval = null;
          if (!window.MathJax && contentRef.current && isMounted) {
            // Fallback to label if MathJax never loads
            contentRef.current.textContent = config.symbol || config.label;
          }
        }, 5000);
        return;
      }

      try {
        // Set innerHTML with LaTeX
        if (contentRef.current && isMounted) {
          contentRef.current.innerHTML = config.latex;
          // Render with MathJax
          await window.MathJax.typesetPromise([contentRef.current]);
        }
      } catch (error) {
        console.error('MathJax rendering error:', error);
        // Fallback to label
        if (contentRef.current && isMounted) {
          contentRef.current.textContent = config.symbol || config.label;
        }
      }
    };

    renderMathJax();

    // Cleanup function
    return () => {
      isMounted = false;
      if (checkInterval) clearInterval(checkInterval);
      if (timeoutId) clearTimeout(timeoutId);
    };
  }, [config.latex, config.symbol, config.label]);

  // Common button styles
  const buttonStyle = {
    fontFamily: 'serif',
    fontSize: '1.1rem',
    minWidth: '48px',
    padding: '8px 12px',
  };

  // Custom SVG (for Matrix Builder button)
  if (config.customSvg) {
    return (
      <Button
        ref={buttonRef}
        variant="secondary"
        onClick={handleClick}
        className="math-btn"
        title={config.tooltip}
        style={buttonStyle}
      >
        {config.customSvg}
      </Button>
    );
  }

  // MathJax-rendered button
  if (config.latex) {
    return (
      <Button
        ref={buttonRef}
        variant="secondary"
        onClick={handleClick}
        className="math-btn"
        title={config.tooltip}
        style={buttonStyle}
      >
        <span ref={contentRef} className="math-btn-content" />
      </Button>
    );
  }

  // Fallback to plain text
  return (
    <Button
      ref={buttonRef}
      variant="secondary"
      onClick={handleClick}
      className="math-btn"
      title={config.tooltip}
      style={buttonStyle}
    >
      {config.symbol || config.label}
    </Button>
  );
});

