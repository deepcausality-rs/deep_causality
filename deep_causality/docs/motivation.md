[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Motivation

The rapid progress in deep learning led to impressive results in several areas such as text to image (Midjourney,
Dall-E), text to speech (Voicemaker), speech recognition (Alexa, Siri), and, more recently, interactive chat
conversations (Chat-GPT). Underling all of these deep learning technologies are a small set of assumptions and concepts.
Namely, the perceptron, the universal approximation theorem, and the independent and identically distributed assumption.

The perceptron remains the primary building block of deep learning that enables much of the observed memory formation.
Fundamentally, a perceptron functions by receiving an input signal and when the signal exceeds the internal threshold,
an activation signal is sent to its output, which in many cases is another perceptron. The actual learning process,
then, means finding the best possible threshold for all perceptrons in a neural net. After the learning process, the
resulting neural net maps input data to defined output for as long as the input data fall within the data distribution
of the learning data set. The careful reader may have noticed that mapping a set of input to another set of output
within a certain data distribution is correlation based.

More precisely, the neural net, often with a high degree of accuracy, estimates how a set of input co-relates to the
trainings output. This process works because of the universal approximation theorem that states that any arbitrary
continuous function can be represented as a combination of linear function. Therefore, a neural net, through adjusting
all its internal thresholds, learns actually a complex combination of linear functions that approximate the function
describing the input data. When the domain of possible input data is very large, so will be the required trainings data
set. These computations on large data, however, require increasingly more time and computational resources. One key
contributor of the massive success of deep learning is the cost-effective availability of commodity hardware and
especially GPU accelerators.

Despite significant progress driven by increasingly larger neuronal nets, several fundamental challenges increasingly
gain more significance. Specifically:

1) Independent and identically distributed (IID) assumption.
2) Contextual agnostic
3) Model blindness

The IID assumption ensures that a model learned of some sample data can either classify or predict similar data. Similar
means that the new data also adhere to the same data distribution and that each data entry is intendent of all the other
data. This ignored any potential relationship between data and, in practice, requires careful feature engineering to
restore potential relations between data.

For example, in timeseries it is perfectly possible that certain observations change disproportionally relative to other
observations when the preceding time interval contained a public holiday. Because of the IID, it is necessary to
de-normalize time information into all its part and annotate each record with additional information such as day of the
week, public holiday, or similar.

Another consequence of the IID is that out of distribution samples simply cannot be recognized. Increasing the sample
size remains one popular way to circumvent this limitation. For very large datasets, re-encoding data into sparse
formats then helps to reduce the unavoidable computational overhead of the large sample size. Lastly, the IID, at its
very core preserves the curve fitting nature of neural nets. Curve fitting means that the learning process consists of
finding the best possible combination of linear function that closely resemble the original function describing the
trainings data. For all application where out-of-sample observations rarely or never occur, all IID based deep learning
remains an attractive solution to classification and prediction problems.

Contextual agnostic means that analyzing and processing data in deep learning occurs without consideration for the
context that generated these data. The strength of the universal approximation theorem clearly results from discarding
all additional information and instead only consider how to best replicate the function of the input data. The theorem
holds if there is no meaningful impact of the context on the data generation. However, when context does impact data
generation, the only feasible way forward really is to collect more data of which many capture effects of the context
largely to restore the ability to approximate the underlying function.

Model blindness refers to the reality that every methodology subject to the IID remains unable to capture causal
relationships because of the stipulated independence of each data record. This is also the reason why contextual
information cannot be considered without additional data wrangling because contextual data would then lead to dependent
variables in a process largely relying on independent data.

These limitations have been described in more detail before, among others in Pearl [Pearl, 2018], and even the military
fully recognizes the reality of these impediments. DARPA writes on its ANSR project
page [ANSR](https://www.darpa.mil/program/assured-neuro-symbolic-learning-and-reasoning):

> "ANSR hypothesizes that several of the limitations in ML today are a consequence of the inability to incorporate
> contextual
> and background knowledge, and treating each data set as an independent, uncorrelated input.
> In the real world, observations are often correlated and a product of an underlying causal mechanism,
> which can be modeled and
> understood" - [ASNR](https://www.darpa.mil/program/assured-neuro-symbolic-learning-and-reasoning)

The key challange can be summarized as:

1) Context & background knowledge
2) Relations between observations
3) Causal mechanism

## Context & background knowledge

There are numerous challenges involved in modelling context that is suitable for general application in machine
learning.

1) Context changes over time.
2) Change of context may happen.
3) Multiple contexts may apply to a model, too.

The first challenge refers to the fact that, as time progresses so progresses the change of the content embedded in a
context. For example, when a car starts driving, other vehicles and pedestrian enter, move through, and leave the
sight of view of the driver. In the case of a self-driving car, computer vision augmented with other sensor data (
i.e. lidar) usually leads to detailed, high resolution context real-time 360 degree field of view. As this augmented
context changes over time, the underlying Ai learning mechanism needs somehow to decide when to intervene and how
compared to when to remain invariant to context changes.

The second challenge occurs when an Ai trained of data within a certain context gets deployed to a different context.
Taking the self-driving car as an example, when the Ai was trained in sunny California, will the same Ai be able to
drive safely on snow roads in Norway during the winter? Likewise, when, for example, a driving assistance Ai was
training on, say, central European roads where road lines and signs are usually clearly visible, will the same Ai
work in a remote African area with only dust roads and without proper road signs? A change of context usually entails
a re-evaluation of the assumption made during the initial training and thus requires a mechanism that tests whether
the underlying assumptions are actually present in the current context.

The third challenge extend the previous two to incorporate information from multiple disjoint contexts. While human
perception occurs simultaneously across different sense hence only suggestion one context at a time, computational
perception relies in multiple data-feed of which each represent a different context. Obviously, one can model an
aggregation layer that merges multiple data-streams into one augmented stream as it the case with, say, lidar
enhanced computer vision in self-driving cars. However, even with aggregation layers in place, in practice there is
more than one data stream to consider at any point in time. In stock trading, for example, the intra-day timeframe
can be considered as a relative short-term context whereas the weekly, monthly, or even yearly timeframe provides a
relative long-term context. At any point in time, if the stock prices remain within a certain range, say the weekly
range, not much is expected, but if for whatever reason, the price falls below, say the lowest low of the month, or
below a certain moving average, significant sale pressure is expected because the general outlook is rather
negative.

In self driving cars, computer vision certainly provides a rather immediate short-term context. Those cars equipped
with a proper front facing radar sensor, receive additional long-term information from a further distance even when
fog clouds conventional vision.

At any point in time, decision must consider the immediate short-term and the relative long-term context to ensure
the right action in the given context. The challenge lies exactly in scale-agnostic context because, in high
frequency stock trading, 5 minutes out is already relatively long-term, whereas in self-driving cars, 5 seconds can
be already too late.

In summary, the content of a context changes over time and hence requires constant updating; next, a change of
context may happen and thus requires a test of whether the assumption made during training remain valid in a context;
lastly, the need to handle multiple contexts emerge from the simple need to handle either multiple timeframes,
multiple ranges of distance, or both.

Considering multiple time-scale invariant contexts during the training and application of advanced machine learning
would contribute to the overall reliability of artificial intelligence because that would enable Ai to recognize and
response to its changing environment.

## Relations between observations

The IID establishes that each data record is independent from all other data records and that all data are identically
distributed. Beginning with the assumption of identical distributed data, it is simply unrealistic to impose such a
restrictive assumption give that, in practice, data collection might be imperfect, incomplete, or simply inconsistent.
Furthermore, when data sampling happens prior to the application of an Ai model, it might happen that different sample
sizes skew the implied data distribution. However, in practice, most modern machine learning algorithms handle shifts in
data distribution relatively well as long as the distribution skew remains within certain limits. A much more pronounced
issue remains handling of out-of-sample data, which in most cases cannot be handled and thus lead to substantial errors.
Increasing the trainings data size somehow reduces chances of encountering out-of-sample errors, but not eliminate
chances entirely.

However, handling relations between data usually requires feature engineering to make implicit relations explicit. For
example, when analyzing a timeseries of sensor data, it may become require adding additional features referring to some
previous data to aid the Ai learning process to comprehend the current data record. This step often increases
classification accuracy i.e. when trying to detect measurement anomalies indicating imminent failure. For example, water
colling systems have a particular series of anomalies i.e. erratic drop in pressure before a pump failure happens.
Therefore, it is often necessary to relate current measurements back to previous one i.e. by calculating the change of a
measurement relative to a moving average to deiced whether and anomaly occurred. In actuality, the data scientist
unwittingly contextualizes the current observation by relating it back to some prior observation; a step that becomes
only necessary in absence of context. Furthermore, even if all relations between attributes and observations somehow
have been accounted for, there is still nothing said about the nature of the relations in absence of a defined causal
mechanism.

## Causal mechanisms

The largest problem, though, remains the assumption of data independence because, in practice, there are many cases of
oblivious causal relations embedded in data streams. These are rarely explicitly encoded in the observed data and as a
result not being accounted for during the trainings phase. For example, in self-driving cars, the augmented vision
system may record how a pedestrian suddenly crosses the street further ahead, forcing the car in front to break
abruptly, which subsequently triggers the emergency break.

It’s inconceivable not to consider the apparent chain of causality and yet the IID requires machine learning algorithm
to look at each video frame and each event individually. It is self-evident that algorithms subject to the IID impose a
non-trivial systematic risk in all areas requiring human safety.
Rather, temporal-spatial causality reasoning combined with, for example, conventional Ai for object detection
and tracking updating a dynamic context would contribute significantly to assert system reliability and safety. 
