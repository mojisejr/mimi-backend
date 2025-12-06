-- Fix JSON Schema Placeholders in Prompts
--
-- This script fixes the template placeholder validation issue where
-- JSON schema examples in prompts are incorrectly identified as placeholders.
--
-- Problem: Database prompts contain JSON examples like {"isValid": boolean, "reason": "..."}
--          The validator finds "isValid" (with quotes) and treats it as placeholder
--          but "isValid" is not in the valid placeholders list.
--
-- Solution: Replace { } in JSON schemas with {{ }} to escape them
--
-- Run this script to fix the template validation errors:
-- psql $DATABASE_URL -f migrations/fix_json_schema_placeholders.sql

-- ============================================================================
-- FIX 1: question_filter prompt - escape JSON schema example
-- ============================================================================

UPDATE prompts
SET prompt_content = REPLACE(prompt_content, '{"isValid": boolean, "reason": "คำอธิบายภาษาไทย"}', '{{"isValid": boolean, "reason": "คำอธิบายภาษาไทย"}}')
WHERE agent_name = 'question_filter'
  AND prompt_content LIKE '%{"isValid": boolean%';

-- ============================================================================
-- FIX 2: reading_agent prompt - escape JSON structure examples
-- ============================================================================

-- Fix the main JSON structure in reading_agent prompt
UPDATE prompts
SET prompt_content = REPLACE(prompt_content,
    '{' || E'\n' || '"header": "คำทักทายและทวนคำถามพร้อมเปิดประเด็นชวนคิด (1 ประโยคในภาษาเดียวกันกับคำถาม)",' || E'\n' || '"cards_reading": [ // รายละเอียดของไพ่ที่ถูกหยิบ',
    '{{' || E'\n' || '"header": "คำทักทายและทวนคำถามพร้อมเปิดประเด็นชวนคิด (1 ประโยคในภาษาเดียวกันกับคำถาม)",' || E'\n' || '"cards_reading": [ // รายละเอียดของไพ่ที่ถูกหยิบ')
WHERE agent_name = 'reading_agent'
  AND prompt_content LIKE '%"header": "คำทักทายและทวนคำถามพร้อมเปิดประเด็นชวนคิด%';

-- Fix the nested card structure in reading_agent prompt
UPDATE prompts
SET prompt_content = REPLACE(prompt_content,
    '{' || E'\n' || '"id": number,' || E'\n' || '"name": "ชื่อไพ่ภาษาอังกฤษ (เช่น The Sun)",',
    '{{' || E'\n' || '"id": number,' || E'\n' || '"name": "ชื่อไพ่ภาษาอังกฤษ (เช่น The Sun)",')
WHERE agent_name = 'reading_agent'
  AND prompt_content LIKE '%"id": number%';

-- Additional safety check: escape any remaining standalone { that might be JSON
UPDATE prompts
SET prompt_content = REPLACE(prompt_content, '{\"isValid\":', '{{\"isValid\":')
WHERE agent_name = 'question_filter'
  AND prompt_content LIKE '{\"isValid\":%';

UPDATE prompts
SET prompt_content = REPLACE(prompt_content, '{\"mood\":', '{{\"mood\":')
WHERE agent_name = 'question_analyzer'
  AND prompt_content LIKE '{\"mood\":%';

-- ============================================================================
-- VERIFICATION: Check the fixes
-- ============================================================================

SELECT
    agent_name,
    LENGTH(prompt_content) as content_length,
    CASE
        WHEN prompt_content LIKE '{{%' THEN '✅ Fixed (uses {{ escape})'
        WHEN prompt_content LIKE '{%}' AND agent_name = 'question_filter' THEN '❌ Still has unescaped {'
        ELSE '✅ OK'
    END as fix_status
FROM prompts
ORDER BY agent_name;

-- ============================================================================
-- TEST: Validate that the prompts can be loaded without validation errors
-- ============================================================================

-- This should return no rows if all JSON schemas are properly escaped
SELECT agent_name, 'Still contains problematic JSON pattern' as issue
FROM prompts
WHERE prompt_content LIKE '{%'
  AND prompt_content NOT LIKE '{{%'
  AND (
    prompt_content LIKE '%{"%' OR  -- Contains quoted JSON keys
    prompt_content LIKE '%{":%' OR -- Contains quoted JSON values
    prompt_content LIKE '%number%' OR -- Contains JSON type hints
    prompt_content LIKE '%boolean%'
  );

COMMIT;