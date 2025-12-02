/**
 * Workflow Advisor System
 * Provides intelligent workflow suggestions based on analysis and project state
 */

class WorkflowAdvisor {
    constructor() {
        this.workflows = {
            quick: {
                name: 'Quick Implementation',
                description: 'Fast track for simple changes and fixes',
                estimatedTime: '5-15 minutes',
                steps: [
                    'Analyze current codebase',
                    'Write minimal tests (if needed)',
                    'Implement changes',
                    'Run validation checks',
                    'Create PR'
                ],
                suitableFor: ['bug fixes', 'small features', 'config changes', 'documentation'],
                command: '/task [description] --quick'
            },
            standard: {
                name: 'Standard Implementation',
                description: 'Balanced approach for typical features',
                estimatedTime: '20-45 minutes',
                steps: [
                    'Create lightweight context',
                    'Generate 2-3 atomic tasks',
                    'Write comprehensive tests',
                    'Implement tasks sequentially',
                    'Integration testing',
                    'Create PR'
                ],
                suitableFor: ['new features', 'API endpoints', 'database changes', 'integrations'],
                command: '/task [description]'
            },
            comprehensive: {
                name: 'Comprehensive Planning',
                description: 'Detailed approach for complex systems',
                estimatedTime: '1-3 hours',
                steps: [
                    'Create detailed context with milestones',
                    'Break into 4+ atomic tasks',
                    'Architecture and security review',
                    'Progressive implementation',
                    'Comprehensive testing',
                    'Performance validation',
                    'Documentation',
                    'Create PR'
                ],
                suitableFor: ['new systems', 'architecture changes', 'major features', 'complex integrations'],
                command: '/task [description] --deep'
            },
            research: {
                name: 'Research & Planning',
                description: 'Exploratory phase for uncertain requirements',
                estimatedTime: '30-60 minutes',
                steps: [
                    'Research requirements',
                    'Prototype proof of concept',
                    'Validate technical approach',
                    'Create implementation plan',
                    'Proceed with appropriate workflow'
                ],
                suitableFor: ['unclear requirements', 'new technologies', 'experimental features'],
                command: '/task [description] --research'
            }
        };

        this.heuristics = {
            complexityIndicators: {
                high: ['system', 'architecture', 'complete', 'full', 'multiple', 'various'],
                medium: ['feature', 'endpoint', 'service', 'component', 'integration'],
                low: ['fix', 'update', 'small', 'minor', 'simple', 'quick']
            },
            riskIndicators: {
                high: ['payment', 'security', 'authentication', 'critical', 'production'],
                medium: ['database', 'external', 'api', 'queue'],
                low: ['ui', 'documentation', 'configuration', 'internal']
            },
            uncertaintyIndicators: [
                'explore', 'research', 'investigate', 'prototype', 'concept', 'proof of concept'
            ]
        };
    }

    /**
     * Analyze request and suggest optimal workflow
     */
    analyzeAndSuggest(description, projectState = {}) {
        const analysis = this.performAnalysis(description, projectState);
        const suggestions = this.generateSuggestions(analysis);
        const recommendations = this.generateRecommendations(analysis, suggestions);

        return {
            analysis,
            suggestions,
            recommendations,
            confidence: this.calculateConfidence(analysis),
            alternativeWorkflows: this.generateAlternatives(analysis)
        };
    }

    /**
     * Perform comprehensive analysis
     */
    performAnalysis(description, projectState) {
        const desc = description.toLowerCase();

        return {
            description,
            complexity: this.assessComplexity(desc),
            risk: this.assessRisk(desc),
            uncertainty: this.assessUncertainty(desc),
            scope: this.assessScope(desc),
            dependencies: this.assessDependencies(desc, projectState),
            projectMaturity: this.assessProjectMaturity(projectState),
            teamCapacity: this.assessTeamCapacity(projectState)
        };
    }

    /**
     * Assess complexity level
     */
    assessComplexity(description) {
        let score = 3; // Base medium score

        // High complexity indicators
        for (const indicator of this.heuristics.complexityIndicators.high) {
            if (description.includes(indicator)) {
                score += 2;
            }
        }

        // Low complexity indicators
        for (const indicator of this.heuristics.complexityIndicators.low) {
            if (description.includes(indicator)) {
                score -= 1;
            }
        }

        // Component count
        const componentWords = ['api', 'database', 'ui', 'queue', 'auth', 'payment'];
        const componentCount = componentWords.filter(word => description.includes(word)).length;
        score += componentCount;

        return this.normalizeScore(score);
    }

    /**
     * Assess risk level
     */
    assessRisk(description) {
        let score = 2; // Base medium-low score

        // High risk indicators
        for (const indicator of this.heuristics.riskIndicators.high) {
            if (description.includes(indicator)) {
                score += 3;
            }
        }

        // Medium risk indicators
        for (const indicator of this.heuristics.riskIndicators.medium) {
            if (description.includes(indicator)) {
                score += 1;
            }
        }

        return this.normalizeScore(score);
    }

    /**
     * Assess uncertainty level
     */
    assessUncertainty(description) {
        let score = 0;

        for (const indicator of this.heuristics.uncertaintyIndicators) {
            if (description.includes(indicator)) {
                score += 3;
            }
        }

        return this.normalizeScore(score);
    }

    /**
     * Assess scope
     */
    assessScope(description) {
        if (description.includes('complete') || description.includes('full') || description.includes('entire')) {
            return 'large';
        }
        if (description.includes('prototype') || description.includes('poc') || description.includes('concept')) {
            return 'experimental';
        }
        if (description.includes('small') || description.includes('minor') || description.includes('quick')) {
            return 'small';
        }
        return 'medium';
    }

    /**
     * Assess dependencies
     */
    assessDependencies(description, projectState) {
        const dependencies = {
            external: [],
            internal: [],
            blockers: []
        };

        // External service dependencies
        const services = ['stripe', 'redis', 'supabase', 'gemini', 'aws', 'google', 'paypal'];
        for (const service of services) {
            if (description.includes(service)) {
                dependencies.external.push(service);
            }
        }

        // Internal dependencies
        if (projectState.existingContexts && projectState.existingContexts.length > 0) {
            dependencies.internal = projectState.existingContexts;
        }

        // Blockers
        if (dependencies.external.length > 2) {
            dependencies.blockers.push('Multiple external service dependencies may require coordination');
        }

        return dependencies;
    }

    /**
     * Assess project maturity
     */
    assessProjectMaturity(projectState) {
        if (!projectState) return 'unknown';

        const indicators = {
            mature: projectState.totalTasks > 20 && projectState.completedTasks > 15,
            growing: projectState.totalTasks > 10 && projectState.completedTasks > 5,
            early: projectState.totalTasks <= 10
        };

        if (indicators.mature) return 'mature';
        if (indicators.growing) return 'growing';
        if (indicators.early) return 'early';
        return 'unknown';
    }

    /**
     * Assess team capacity
     */
    assessTeamCapacity(projectState) {
        if (!projectState) return 'unknown';

        // Simple heuristic based on recent task completion rate
        if (projectState.recentTasks && projectState.recentTasks.length > 0) {
            const avgTime = projectState.recentTasks.reduce((sum, task) => sum + task.estimatedTime, 0) / projectState.recentTasks.length;

            if (avgTime < 20) return 'high';
            if (avgTime < 40) return 'medium';
            return 'low';
        }

        return 'medium';
    }

    /**
     * Normalize score to 1-10 scale
     */
    normalizeScore(score) {
        return Math.max(1, Math.min(10, Math.round(score)));
    }

    /**
     * Generate workflow suggestions
     */
    generateSuggestions(analysis) {
        const suggestions = [];

        // Primary suggestion based on analysis
        let primaryWorkflow = this.determinePrimaryWorkflow(analysis);
        suggestions.push({
            workflow: primaryWorkflow,
            confidence: this.calculateWorkflowConfidence(analysis, primaryWorkflow),
            reasoning: this.explainWorkflowChoice(analysis, primaryWorkflow)
        });

        // Alternative suggestions
        const alternatives = this.generateAlternativeSuggestions(analysis);
        suggestions.push(...alternatives);

        return suggestions;
    }

    /**
     * Determine primary workflow
     */
    determinePrimaryWorkflow(analysis) {
        // High uncertainty → research workflow
        if (analysis.uncertainty >= 7) {
            return 'research';
        }

        // High complexity + high risk → comprehensive
        if (analysis.complexity >= 7 && analysis.risk >= 7) {
            return 'comprehensive';
        }

        // High risk alone → comprehensive (for safety)
        if (analysis.risk >= 8) {
            return 'comprehensive';
        }

        // Low complexity + low risk → quick
        if (analysis.complexity <= 3 && analysis.risk <= 3) {
            return 'quick';
        }

        // Default to standard
        return 'standard';
    }

    /**
     * Calculate confidence in workflow recommendation
     */
    calculateWorkflowConfidence(analysis, workflow) {
        let confidence = 0.7; // Base confidence

        // Higher confidence for clear signals
        if (workflow === 'research' && analysis.uncertainty >= 8) confidence += 0.2;
        if (workflow === 'comprehensive' && analysis.complexity >= 8 && analysis.risk >= 8) confidence += 0.2;
        if (workflow === 'quick' && analysis.complexity <= 2 && analysis.risk <= 2) confidence += 0.2;
        if (workflow === 'standard' && analysis.complexity >= 4 && analysis.complexity <= 6) confidence += 0.1;

        return Math.min(1.0, confidence);
    }

    /**
     * Explain workflow choice
     */
    explainWorkflowChoice(analysis, workflow) {
        const reasons = [];

        switch (workflow) {
            case 'research':
                if (analysis.uncertainty >= 7) reasons.push('High uncertainty detected in requirements');
                reasons.push('Exploratory phase needed to validate technical approach');
                break;

            case 'comprehensive':
                if (analysis.complexity >= 7) reasons.push('High complexity requires detailed planning');
                if (analysis.risk >= 7) reasons.push('High risk factors need comprehensive approach');
                if (analysis.dependencies.external.length > 2) reasons.push('Multiple external dependencies require coordination');
                break;

            case 'quick':
                if (analysis.complexity <= 3) reasons.push('Low complexity suggests simple implementation');
                if (analysis.risk <= 3) reasons.push('Low risk allows fast tracking');
                break;

            case 'standard':
                reasons.push('Balanced approach for typical feature development');
                if (analysis.complexity >= 4 && analysis.complexity <= 6) reasons.push('Moderate complexity suits standard workflow');
                break;
        }

        return reasons.join('; ');
    }

    /**
     * Generate alternative suggestions
     */
    generateAlternativeSuggestions(analysis) {
        const alternatives = [];
        const primaryWorkflow = this.determinePrimaryWorkflow(analysis);

        // Suggest quick alternative for non-critical work
        if (primaryWorkflow !== 'quick' && analysis.risk <= 4) {
            alternatives.push({
                workflow: 'quick',
                confidence: 0.4,
                reasoning: 'Lower risk approach if you prefer rapid iteration'
            });
        }

        // Suggest comprehensive alternative for critical work
        if (primaryWorkflow !== 'comprehensive' && analysis.risk >= 6) {
            alternatives.push({
                workflow: 'comprehensive',
                confidence: 0.5,
                reasoning: 'More thorough approach recommended for higher risk work'
            });
        }

        return alternatives;
    }

    /**
     * Generate actionable recommendations
     */
    generateRecommendations(analysis, suggestions) {
        const primarySuggestion = suggestions[0];
        const workflow = this.workflows[primarySuggestion.workflow];

        const recommendations = {
            primary: {
                command: workflow.command.replace('[description]', analysis.description),
                workflow: workflow.name,
                estimatedTime: workflow.estimatedTime,
                confidence: primarySuggestion.confidence
            },
            preparation: this.generatePreparationSteps(analysis),
            warnings: this.generateWarnings(analysis),
            tips: this.generateTips(analysis, primarySuggestion.workflow)
        };

        return recommendations;
    }

    /**
     * Generate preparation steps
     */
    generatePreparationSteps(analysis) {
        const steps = ['Review current codebase structure'];

        if (analysis.dependencies.external.length > 0) {
            steps.push(`Verify external service access: ${analysis.dependencies.external.join(', ')}`);
            steps.push('Run `/test-env` to validate service connectivity');
        }

        if (analysis.risk >= 6) {
            steps.push('Review security implications');
            steps.push('Plan rollback strategy');
        }

        if (analysis.scope === 'large') {
            steps.push('Break down into smaller milestones');
        }

        return steps;
    }

    /**
     * Generate warnings
     */
    generateWarnings(analysis) {
        const warnings = [];

        if (analysis.dependencies.external.length > 2) {
            warnings.push('Multiple external service dependencies may cause delays');
        }

        if (analysis.risk >= 8) {
            warnings.push('High risk work - consider comprehensive planning');
        }

        if (analysis.uncertainty >= 6) {
            warnings.push('Uncertain requirements - research phase recommended');
        }

        return warnings;
    }

    /**
     * Generate tips
     */
    generateTips(analysis, workflow) {
        const tips = [];

        // General tips
        tips.push('Write tests first (TDD) to ensure quality');
        tips.push('Use `/test-env` to validate external services');

        // Workflow-specific tips
        switch (workflow) {
            case 'quick':
                tips.push('Focus on core functionality only');
                tips.push('Add comprehensive tests for edge cases');
                break;

            case 'comprehensive':
                tips.push('Plan for monitoring and logging');
                tips.push('Consider performance implications');
                break;

            case 'research':
                tips.push('Document findings and decisions');
                tips.push('Create proof of concept before full implementation');
                break;
        }

        // Risk-specific tips
        if (analysis.risk >= 6) {
            tips.push('Implement proper error handling and monitoring');
        }

        return tips;
    }

    /**
     * Calculate overall confidence
     */
    calculateConfidence(analysis) {
        let factors = [];

        // High confidence factors
        if (analysis.complexity <= 3 || analysis.complexity >= 7) {
            factors.push(0.2); // Clear complexity signal
        }
        if (analysis.risk <= 3 || analysis.risk >= 7) {
            factors.push(0.2); // Clear risk signal
        }
        if (analysis.uncertainty === 0 || analysis.uncertainty >= 7) {
            factors.push(0.2); // Clear uncertainty signal
        }

        // Penalize ambiguity
        if (analysis.complexity >= 4 && analysis.complexity <= 6 &&
            analysis.risk >= 4 && analysis.risk <= 6) {
            factors.push(-0.1); // Ambiguous signals
        }

        const baseConfidence = 0.6;
        const adjustment = factors.reduce((sum, factor) => sum + factor, 0);

        return Math.max(0.3, Math.min(0.95, baseConfidence + adjustment));
    }

    /**
     * Generate alternative workflows
     */
    generateAlternatives(analysis) {
        const alternatives = [];

        // Always include all workflows as options
        for (const [key, workflow] of Object.entries(this.workflows)) {
            alternatives.push({
                name: workflow.name,
                command: workflow.command.replace('[description]', analysis.description),
                suitable: workflow.suitableFor,
                estimatedTime: workflow.estimatedTime,
                whenToUse: this.getWhenToUse(key, analysis)
            });
        }

        return alternatives;
    }

    /**
     * Get when to use guidance for workflow
     */
    getWhenToUse(workflow, analysis) {
        switch (workflow) {
            case 'quick':
                return analysis.complexity <= 4 ? 'Perfect match for your requirements' : 'If you prefer rapid iteration';
            case 'standard':
                return analysis.complexity >= 4 && analysis.complexity <= 6 ? 'Ideal for your requirements' : 'If you want balanced approach';
            case 'comprehensive':
                return analysis.risk >= 6 ? 'Recommended for your high-risk work' : 'If you need thorough planning';
            case 'research':
                return analysis.uncertainty >= 5 ? 'Recommended for uncertain requirements' : 'If you need exploration';
            default:
                return 'Alternative approach';
        }
    }
}

module.exports = WorkflowAdvisor;