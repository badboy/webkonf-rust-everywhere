#!/usr/bin/env ruby
# encoding: utf-8

require "json"
require "syro"
require "ohm"
require "nobi"

require_relative "models"

FRONTENT_DIR = "../frontend"
SIGNER = Nobi::Signer.new('a3f1194a5a26a1dc1730ad2df7d870f9e11194aba6c08865c38f2a2b65bb8a4b')



class API < Syro::Deck
  def log *args
    $stderr.puts "@@@@ %s" % args.inspect
  end

  def default_headers
    { "Content-Type" => "application/json" }
  end

  def json(object)
    res.write object.to_json
  end

  def current_user
    signed_user_id = req.cookies["user-id"]
    if signed_user_id.nil?
      return NullUser.new
    end

    begin
      user_id = SIGNER.unsign(signed_user_id)
      u = User.with(:name, user_id)
      u 
    rescue Nobi::BadData
      NullUser.new
    end
  end

  def set_user_cookie(user_id)
    signed_user_id = SIGNER.sign(user_id)
    res.set_cookie("user-id", {
      path: "/",
      value: signed_user_id
    })
  end
end

app = Syro.new(API) {
  on("time") {
    on("login") {
      get {
        user_id = req.cookies["user-id"]
        if user_id.nil? || user_id.empty?
          retries = 5
          begin
            user_id = rand(100_000_000).to_s
            u = User.create(name: user_id)
          rescue Ohm::UniqueIndexViolation
            retries -= 1

            if retries == 0
              retries = -1
              res.delete_cookie("user-id")
              res.status = 401
              json(authorized: false, reason: "failed to create user")
            else
              retry
            end
          end

          if retries != -1
            set_user_cookie(user_id)
            json(authorized: true, user_id: user_id, user: u.id)
          end
        else
          begin
            u = current_user
            $stderr.puts "CURRENT USER: %s" % u.inspect
            if u.nil?
              res.delete_cookie("user-id")
              res.status = 401
              json(authorized: false, reason: "No such user")
            else
              set_user_cookie(u.name)
              json(authorized: true, user_id: user_id, user: u.id)
            end
          rescue Nobi::BadData
            res.delete_cookie("user-id")
            res.status = 401
            json(authorized: false, reason: "Can't validate")
          end
        end
      }
    }

    on("new") {
      post {
        u = current_user
        if u.nil?
          res.delete_cookie("user-id")
          res.status = 401
          json(authorized: false, reason: "No such user")
          return
        end

        start = req.params["start"].to_i
        stop = req.params["stop"].to_i
        if start == 0 || stop == 0
          json(error: "Start and stop parameters required.")
        else
          track = TimeTrack.create start: start, stop: stop, user: u
          json(track)
        end
      }
    }

    on(:track_id) {
      get {
        id = inbox[:track_id].to_i
        track = TimeTrack[id]
        json(track)
      }
    }

    get {
      json(current_user.tracks.to_a)
    }
  }
}

if ENV.fetch("RACK_ENV") == "production"
  mount_path = "/legacy"
else
  mount_path = "/"
end

map mount_path do
  map "/api" do
    run(app)
  end

  use Rack::Session::Cookie, secret: "87998b9378250664e13e1f5d5922856391fae867f6d5c869e0721cb867ad1437"
  run Rack::File.new(FRONTENT_DIR)
end
