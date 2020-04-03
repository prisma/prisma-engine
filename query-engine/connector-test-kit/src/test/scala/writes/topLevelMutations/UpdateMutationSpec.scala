package writes.topLevelMutations

import org.scalatest.{FlatSpec, Matchers}
import play.api.libs.json.Json
import util._

class UpdateMutationSpec extends FlatSpec with Matchers with ApiSpecBase {

  "The Update Mutation" should "update an item" in {
    val project = ProjectDsl.fromString {
      """
        |model ScalarModel {
        |  id          String  @id @default(cuid())
        |  optString   String?
        |  optInt      Int?
        |  optFloat    Float?
        |  optBoolean  Boolean?
        |  optEnum     MyEnum?
        |  optDateTime DateTime?
        |}
        |
        |enum MyEnum {
        |  A
        |  ABCD
        |}
      """.stripMargin
    }
    database.setup(project)

    val createResult = server.query(
      """mutation {
        |  createScalarModel(data: {
        |  })
        |  { id }
        |}""",
      project = project
    )
    val id = createResult.pathAsString("data.createScalarModel.id")

    val updateResult = server.query(
      s"""
        |mutation {
        |  updateScalarModel(
        |    data:{
        |      optString: "lala${TroubleCharacters.value}", optInt: 1337, optFloat: 1.234, optBoolean: true, optEnum: A, optDateTime: "2016-07-31T23:59:01.000Z"
        |    }
        |    where: {
        |      id: "$id"
        |    }
        |  ){
        |    optString, optInt, optFloat, optBoolean, optEnum, optDateTime
        |  }
        |}
      """.stripMargin,
      project
    )

    updateResult.pathAsJsValue("data.updateScalarModel") should be(Json.parse(
      s"""{"optString":"lala${TroubleCharacters.value}","optInt":1337,"optFloat":1.234,"optBoolean":true,"optEnum":"A","optDateTime":"2016-07-31T23:59:01.000Z"}"""))

    val query = server.query(
      s"""
         |{
         |  scalarModels {
         |    id
         |  }
         |}
       """.stripMargin,
      project
    )
    query.pathAsJsValue("data.scalarModels").toString should equal(s"""[{"id":"$id"}]""")
  }

  "The Update Mutation" should "update an item by a unique field" in {
    val project = ProjectDsl.fromString {
      """
        |model Todo {
        |  id    String  @id @default(cuid())
        |  title String
        |  alias String? @unique
        |}
      """.stripMargin
    }
    database.setup(project)

    val alias = "the-alias"
    server.query(
      s"""
        |mutation {
        |  createTodo(
        |    data: {
        |      title: "initial title", alias: "$alias"
        |    }
        |  ){
        |    id
        |  }
        |}
      """.stripMargin,
      project
    )

    val updateResult = server.query(
      s"""
        |mutation {
        |  updateTodo(
        |    data: {
        |      title: "updated title"
        |    }
        |    where: {
        |      alias: "$alias"
        |    }
        |  ){
        |    title
        |  }
        |}""".stripMargin,
      project
    )
    updateResult.pathAsString("data.updateTodo.title") should equal("updated title")
  }

  "The Update Mutation" should "gracefully fail when trying to update an item by a unique field with a non-existing value" in {
    val project = ProjectDsl.fromString {
      """
        |model Todo {
        |  id     String  @id @default(cuid())
        |  title  String
        |  alias  String? @unique
        |}
      """.stripMargin
    }
    database.setup(project)

    val alias = "the-alias"
    server.query(
      s"""
         |mutation {
         |  createTodo(
         |    data: {
         |      title: "initial title", alias: "$alias"
         |    }
         |  ){
         |    id
         |  }
         |}
      """.stripMargin,
      project
    )

    server.queryThatMustFail(
      s"""
         |mutation {
         |  updateTodo(
         |    data: {
         |      title: "updated title"
         |    }
         |    where: {
         |      alias: "NOT A VALID ALIAS"
         |    }
         |  ){
         |    title
         |  }
         |}""".stripMargin,
      project,
      errorCode = 2016,
      errorContains = """Query interpretation error. Error for binding '0': RecordNotFound(\"Record to update not found.\"""
    )
  }

  "Updating" should "change the updatedAt datetime" in {
    val project = ProjectDsl.fromString {
      """
        |model Todo {
        |  id     String  @id @default(cuid())
        |  title  String?
        |  alias  String? @unique
        |  text   String?
        |  createdAt DateTime @default(now())
        |  updatedAt DateTime @updatedAt
        |}
      """.stripMargin
    }
    database.setup(project)

    val alias = "the-alias"
    server.query(
      s"""
         |mutation {
         |  createTodo(
         |    data: {
         |      title: "initial title",
         |      text: "some text"
         |      alias: "$alias"
         |    }
         |  ){
         |    createdAt
         |    updatedAt
         |  }
         |}
      """,
      project
    )

    Thread.sleep(1000)

    val res = server.query(
      s"""
         |mutation {
         |  updateTodo(
         |    data: {
         |      title: null
         |    }
         |    where: {
         |      alias: "$alias"
         |    }
         |  ){
         |    createdAt
         |    updatedAt
         |  }
         |}""",
      project
    )

    val createdAt = res.pathAsString("data.updateTodo.createdAt")
    val updatedAt = res.pathAsString("data.updateTodo.updatedAt")

    createdAt should not be updatedAt
  }

  "UpdatedAt and createdAt" should "be mutable with an update" in {
    val project = ProjectDsl.fromString {
      """
        |model User {
        |  id        String   @id @default(cuid())
        |  name      String
        |  createdAt DateTime @default(now())
        |  updatedAt DateTime @updatedAt
        |}
      """.stripMargin
    }
    database.setup(project)

    val userId = server
      .query(
        s"""mutation {
         |  createUser(
         |    data:{
         |      name: "Staplerfahrer Klaus"
         |    }
         |  ){
         |    id
         |  }
         |}""".stripMargin,
        project = project
      )
      .pathAsString("data.createUser.id")

    val res = server
      .query(
        s"""mutation {
         |  updateUser(
         |    where: {
         |      id: "$userId"
         |    }
         |    data:{
         |      createdAt: "2000-01-01T00:00:00Z"
         |      updatedAt: "2001-01-01T00:00:00Z"
         |    }
         |  ){
         |    createdAt
         |    updatedAt
         |  }
         |}""".stripMargin,
        project = project
      )

    // We currently have a datetime precision of 3, so Prisma will add .000
    res.pathAsString("data.updateUser.createdAt") should be("2000-01-01T00:00:00.000Z")
    res.pathAsString("data.updateUser.updatedAt") should be("2001-01-01T00:00:00.000Z")
  }
}
