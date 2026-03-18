/**
 * Hook for Z3 verification and satisfiability checking
 */

import { useState, useCallback } from 'react';
import type { EditorNode } from '../types/ast';
import { verify, checkSat } from '../api/kleis';
import type { VerifyResponse, CheckSatResponse } from '../api/kleis';

export function useVerify() {
  const [verifying, setVerifying] = useState(false);
  const [verifyResult, setVerifyResult] = useState<VerifyResponse | null>(null);
  const [verifyError, setVerifyError] = useState<string | null>(null);

  const [checkingSat, setCheckingSat] = useState(false);
  const [satResult, setSatResult] = useState<CheckSatResponse | null>(null);
  const [satError, setSatError] = useState<string | null>(null);

  const doVerify = useCallback(async (ast: EditorNode) => {
    setVerifying(true);
    setVerifyError(null);
    
    const result = await verify(ast);
    setVerifyResult(result);
    
    if (!result.success) {
      setVerifyError(result.error || 'Verification failed');
    }
    
    setVerifying(false);
    return result;
  }, []);

  const doCheckSat = useCallback(async (ast: EditorNode) => {
    setCheckingSat(true);
    setSatError(null);
    
    const result = await checkSat(ast);
    setSatResult(result);
    
    if (!result.success) {
      setSatError(result.error || 'Satisfiability check failed');
    }
    
    setCheckingSat(false);
    return result;
  }, []);

  return {
    verify: doVerify,
    checkSat: doCheckSat,
    verifying,
    verifyResult,
    verifyError,
    checkingSat,
    satResult,
    satError,
  };
}

