I need you to research the best way to connect orbit to be able to interact with the DAW software "ableton live".

you need to first start with reading and analyzing the codebase, start with the crates, and do specifically look at the orbit connector crate where the code for this will sit.

to connect to ableton live we got multiple options available to us:
- an ableton mcp server I found on github could be leveraged: https://github.com/ahujasid/ableton-mcp . It sounds promissing, but I'd rather leverage their code, copy and improve it, instead of just using their library. 
- ableton live max api: (https://help.ableton.com/hc/en-us/articles/5402681764242-Controlling-Live-using-Max-for-Live, https://docs.cycling74.com/legacy/max8/vignettes/live_api_overview, https://www.ableton.com/en/live/max-for-live/). It sounds like this is purely for max, rather than the entire live platform but I could be wrong. please look into that.
- and there's ableton OCS or whatever (https://www.ableton.com/en/packs/connection-kit/)

we're using rust and python through pyo3, so we'll use rust where we can and only python where we need to such as python libraries ai or ableton api related