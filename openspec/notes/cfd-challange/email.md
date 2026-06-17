The cfd.university community challenge
Hi Marvin



The first iPhone was introduced in 2007 and, almost 20 years later, it is pretty difficult to get a mobile that isn't a smartphone and inspired by the original smartphone concept of the iPhone.



Yet, in 2013, I was still running around with my Nokia 6111 and I was quite happy with it. It had much better battery life than any comparable "modern phone", and so, I kept with it for years.



But then I met the Koreans, and they were actively disturbed by the brick I was carrying around, and they successfully bullied me into buying a Samsung phone (of course!). Now I had worse battery life, snake2 was gone, but I was allowed to be seen with the Koreans in public.



According to Simon Sinek's law of diffusion of innovation, I am at the far-right tail of picking up technologies, i.e. a laggard. I'm the person signing up for Facebook when grandparents have realised that grandchildren are no longer on there, and who knows, maybe one day I'll start to show an interest in crypto ...



And so, given how quickly large language models like ChatGPT were spreading, it took me quite some time to even try it. Once I did, I jumped on the hype train, tested it for CFD development and wrote my article on whether ChatGPT can write functional CFD solvers or not. It couldn't at least, that was my conclusion.



If you want to mock me for the article, you probably have ample justification. In fact, some of you did, and reached out after I published the article, and I was never quite sure if I should keep it or just throw away.



I have decided to strike a middle-ground, and provided an updated view, which most likely will age even worse than the previous article.



I no longer believe that LLMs cannot write CFD solver, in-fact, with the advances made of agentic AI, I believe we are just seeing the beginning of another industrial revolution, this time not automating manual labour but rather knowledge work.



If we look at what the industrial revolution has made possible (just look at the transport sector, e.g. the automotive and aerospace), there are untapped potentials out there that were prohibitively expensive before, but can now be tackled by a few agents doodling around in the background (as I write this email, I have an agent debug one of my OpenFOAM solvers in the background which is fascinating to watch in real time).



I am trying to get ahead of the innovation curve, but given the amount of books already written on agentic AI (interestingly using agents themselves!), I am probably still a laggard.



In any case, I thought, instead of just updating my article and writing opinions that no one cares about or that will have become outdated by the time the article hits the press, I thought, let's have some fun today, and I invite you to participate in the cfd.university community challenge!

The challenge
Full details can be found here:



https://github.com/cfd-university/the-cfd.university-challenge-2026



I thought, let's put LLMs to the test and see how good their CFD solvers are. Sure, I can prompt them myself and see how well they do, but I may approach this with very specific constraints.



Instead, here is what I propose: I have written a small and simple CFD solver for a very narrow problem. It is so narrow that no LLM should be struggling to write a CFD solver for it. The challenge is not whether it can write a solver, but whether it can write a CFD solver that is comparable to a handcrafted solver.



The solver I have written is using all the tricks I can think of to make the solver fast and accurate, based on my collected experience of writing CFD solvers and reading the literature. I want to see if LLMs can get to the same point.



The metrics we will observe are the accuracy and the speed, i.e. how quickly it converges, and how close we get to the reference data.



Anyone can participate! If you feel you have absolutely no idea how to even get started writing a CFD solver, or you are a seasoned expert and have written CFD solvers your entire life, or you are anywhere in between on this spectrum, you can participate.



I am particularly interested to see how LLMs perform and how your background (i.e. the level of experience) influences the result. However, if you feel offended by vibe-coding or AI-engineering, that's cool, too. I am also interested in seeing if your handcrafted CFD solver can beat mine.



The challenge opens today and is running for 4 weeks until the 7th of July 2026. You can either submit your pull requests to me or code by email.



Once finished, I will go through the results and publish my findings with you. There will also be a leaderboard, showing who produced the fastest and most accurate code!



Full instructions on how to participate and what to submit can be found on the dedicated GitHub repository: 


I was planning for a long time to do some project together, joint vibe coding wasn't on that list, but I thought this would be a good point to start, as it is very inclusive and anyone can participate. I hope to see you and your submission soon!

The role of AI in for CFD solver development

Ah, yes, I almost forgot, here is the updated article which will contain some additional thoughts on the current state of AI and how that affects CFD, as well as the details of the challenge (which you can also find on the Github repository directly).

https://cfd.university/blog//can-chatgpt-write-fully-functional-cfd-solvers

All the best!

Tom