FROM node:14-alpine as build

WORKDIR /usr/src/cdl/

COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build


FROM nginx
COPY --from=build /usr/src/cdl/public  /usr/share/nginx/html