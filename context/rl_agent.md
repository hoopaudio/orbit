https://arxiv.org/abs/2508.14040

ComputerRL: Scaling End-to-End Online Reinforcement Learning for Computer Use Agents
Hanyu Lai, Xiao Liu, Yanxiao Zhao, Han Xu, Hanchen Zhang, Bohao Jing, Yanyu Ren, Shuntian Yao, Yuxiao Dong, Jie Tang
We introduce ComputerRL, a framework for autonomous desktop intelligence that enables agents to operate complex digital workspaces skillfully. ComputerRL features the API-GUI paradigm, which unifies programmatic API calls and direct GUI interaction to address the inherent mismatch between machine agents and human-centric desktop environments. Scaling end-to-end RL training is crucial for improvement and generalization across diverse desktop tasks; however, it remains challenging due to environmental inefficiency and instability during extended training. To support scalable and robust training, we develop a distributed RL infrastructure capable of orchestrating thousands of parallel virtual desktop environments to accelerate large-scale online RL. Furthermore, we propose Entropulse, a training strategy that alternates reinforcement learning with supervised fine-tuning, effectively mitigating entropy collapse during extended training runs. We employ ComputerRL on open models GLM-4-9B-0414 and GLM-4.1V-9B-Thinking, and evaluate them on the OSWorld benchmark. The AutoGLM-OS-9B achieves a new state-of-the-art accuracy of 48.9%, demonstrating significant improvements for general agents in desktop automation. Our code and the new OfficeWorld benchmark are available at this https URL. The algorithm and framework are adopted in building AutoGLM (Liu et al., 2024b).



https://github.com/trycua/acu


https://openpipe.ai/blog/art-e-mail-agent
https://github.com/OpenPipe/ART
https://openpipe.ai/blog/serverless-rl
https://art.openpipe.ai/integrations/langgraph-integration