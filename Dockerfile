FROM alpine

RUN mkdir /lttpoll-com

COPY ./assets /lttpoll-com/assets
COPY ./templates /lttpoll-com/templates
COPY ./lttpoll-com /lttpoll-com/lttpoll-com

WORKDIR /lttpoll-com

CMD [ "/lttpoll-com/lttpoll-com" ]
